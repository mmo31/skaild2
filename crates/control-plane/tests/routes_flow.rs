use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::json;
use serial_test::file_serial;
use tower::util::ServiceExt;
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions::MemoryStore;
use time::Duration;

async fn create_test_app() -> (axum::Router, sqlx::PgPool) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://skaild2:skaild2_dev_password@localhost:5432/skaild2_test".to_string());

    let config = shared::config::AppConfig::new(database_url);
    let pool = shared::db::create_pool(&config).await.expect("Failed to create pool");
    shared::db::run_migrations(&pool).await.expect("Failed to run migrations");

    // Clean in FK-safe order: routes before applications
    sqlx::query("DELETE FROM routes").execute(&pool).await.expect("Failed to clean routes");
    sqlx::query("DELETE FROM applications").execute(&pool).await.expect("Failed to clean applications");
    sqlx::query("DELETE FROM admins").execute(&pool).await.expect("Failed to clean admins");

    shared::create_admin(
        &pool,
        shared::CreateAdminInput {
            name: "Test Admin".to_string(),
            email: "admin@example.com".to_string(),
            password: "SecurePassword123!".to_string(),
        },
    )
    .await
    .expect("Failed to create test admin");

    let state = control_plane::state::AppState::new(pool.clone(), reqwest::Client::new());
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600)));

    let app = axum::Router::new()
        .route("/api/auth/login",  axum::routing::post(control_plane::api::login))
        .route("/api/auth/logout", axum::routing::post(control_plane::api::logout))
        .route("/api/applications",
               axum::routing::get(control_plane::api::list_applications)
               .post(control_plane::api::create_application))
        .route("/api/applications/:id",
               axum::routing::get(control_plane::api::get_application)
               .put(control_plane::api::update_application))
        .route("/api/applications/:app_id/routes",
               axum::routing::get(control_plane::api::list_routes)
               .post(control_plane::api::create_route))
        .route("/api/routes/:id",
               axum::routing::get(control_plane::api::get_route)
               .put(control_plane::api::update_route))
        .route("/api/internal/routes", axum::routing::get(control_plane::api::get_internal_routes))
        .layer(session_layer)
        .with_state(state);

    (app, pool)
}

async fn login_and_get_cookie(app: &axum::Router) -> String {
    let payload = json!({ "email": "admin@example.com", "password": "SecurePassword123!" });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "login failed");
    resp.headers()
        .get(header::SET_COOKIE)
        .expect("expected Set-Cookie header")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

async fn create_test_application(app: &axum::Router, cookie: &str) -> String {
    let payload = json!({
        "name": "Test App",
        "upstream_url": "http://192.168.1.10:8080",
        "hostname": "app.example.com"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/applications")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie)
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED, "create application failed");
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    json["id"].as_str().unwrap().to_string()
}

#[tokio::test]
#[file_serial]
async fn test_create_route_requires_auth() {
    let (app, _pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;
    let app_id = create_test_application(&app, &cookie).await;

    let payload = json!({
        "host": "app.example.com",
        "access_mode": "login_required"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/applications/{app_id}/routes"))
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                // No cookie — should be 401
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[file_serial]
async fn test_create_route_persists() {
    let (app, pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;
    let app_id = create_test_application(&app, &cookie).await;

    let payload = json!({
        "host": "app.example.com",
        "path_prefix": "/api",
        "access_mode": "login_required"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/applications/{app_id}/routes"))
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["host"], "app.example.com");
    assert_eq!(json["path_prefix"], "/api");
    assert_eq!(json["access_mode"], "login_required");
    assert_eq!(json["enabled"], true);
    assert!(json["id"].is_string());

    // Confirm persisted in DB
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM routes")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 1);
}

#[tokio::test]
#[file_serial]
async fn test_list_routes_returns_created() {
    let (app, _pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;
    let app_id = create_test_application(&app, &cookie).await;

    // Create a route
    let payload = json!({
        "host": "app.example.com",
        "access_mode": "public"
    });
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/applications/{app_id}/routes"))
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // List routes
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/applications/{app_id}/routes"))
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().expect("expected array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["access_mode"], "public");
    assert_eq!(arr[0]["host"], "app.example.com");
}

#[tokio::test]
#[file_serial]
async fn test_update_route_changes_access_mode() {
    let (app, _pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;
    let app_id = create_test_application(&app, &cookie).await;

    // Create route with login_required
    let payload = json!({
        "host": "app.example.com",
        "access_mode": "login_required"
    });
    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/applications/{app_id}/routes"))
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body = to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let route_id = created["id"].as_str().unwrap().to_string();

    // Update to public
    let update = json!({ "access_mode": "public" });
    let put_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/routes/{route_id}"))
                .method("PUT")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(serde_json::to_vec(&update).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(put_resp.status(), StatusCode::OK);
    let body = to_bytes(put_resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["access_mode"], "public");

    // GET to confirm persisted
    let get_resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/routes/{route_id}"))
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let body = to_bytes(get_resp.into_body(), usize::MAX).await.unwrap();
    let got: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(got["access_mode"], "public");
}

#[tokio::test]
#[file_serial]
async fn test_internal_routes_endpoint_no_auth() {
    let (app, _pool) = create_test_app().await;

    // No cookie required for internal endpoint
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/internal/routes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.is_array());
}
