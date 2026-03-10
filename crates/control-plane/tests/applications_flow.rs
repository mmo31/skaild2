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

    sqlx::query("DELETE FROM applications").execute(&pool).await.expect("Failed to clean applications");
    sqlx::query("DELETE FROM admins").execute(&pool).await.expect("Failed to clean admins");

    // Create a test admin
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

    let state = control_plane::state::AppState::new(pool.clone());
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600)));

    let app = axum::Router::new()
        .route("/api/setup/create-admin", axum::routing::post(control_plane::api::create_first_admin))
        .route("/api/auth/login",  axum::routing::post(control_plane::api::login))
        .route("/api/auth/logout", axum::routing::post(control_plane::api::logout))
        .route("/api/applications",     axum::routing::get(control_plane::api::list_applications)
                                              .post(control_plane::api::create_application))
        .route("/api/applications/:id", axum::routing::get(control_plane::api::get_application)
                                              .put(control_plane::api::update_application))
        .layer(session_layer)
        .with_state(state);

    (app, pool)
}

/// Log in with the pre-created admin and return the session cookie string.
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
        .expect("expected Set-Cookie")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
#[file_serial]
async fn test_create_application_requires_auth() {
    let (app, _pool) = create_test_app().await;

    let payload = json!({
        "name": "Home Assistant",
        "upstream_url": "http://192.168.1.10:8123",
        "hostname": "ha.example.com"
    });

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/applications")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[file_serial]
async fn test_list_applications_requires_auth() {
    let (app, _pool) = create_test_app().await;

    let resp = app
        .oneshot(Request::builder().uri("/api/applications").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[file_serial]
async fn test_create_application_persists() {
    let (app, pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;

    let payload = json!({
        "name": "Home Assistant",
        "upstream_url": "http://192.168.1.10:8123",
        "hostname": "ha.example.com"
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/applications")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie.clone())
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "Home Assistant");
    assert_eq!(json["hostname"], "ha.example.com");
    assert_eq!(json["upstream_url"], "http://192.168.1.10:8123");
    assert_eq!(json["enabled"], true);
    assert!(json["id"].is_string());

    // Verify persisted in DB
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM applications")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 1);
}

#[tokio::test]
#[file_serial]
async fn test_list_applications_returns_created() {
    let (app, _pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;

    // Create one
    let payload = json!({
        "name": "Grafana",
        "upstream_url": "http://192.168.1.20:3000",
        "hostname": "grafana.example.com"
    });
    let _ = app.clone().oneshot(
        Request::builder()
            .uri("/api/applications")
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, cookie.clone())
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap(),
    ).await.unwrap();

    // List
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/applications")
                .header(header::COOKIE, cookie.clone())
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
    assert_eq!(arr[0]["hostname"], "grafana.example.com");
}

#[tokio::test]
#[file_serial]
async fn test_update_application_changes_values() {
    let (app, _pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;

    // Create
    let payload = json!({
        "name": "Portainer",
        "upstream_url": "http://192.168.1.5:9000",
        "hostname": "portainer.example.com"
    });
    let create_resp = app.clone().oneshot(
        Request::builder()
            .uri("/api/applications")
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, cookie.clone())
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap(),
    ).await.unwrap();

    let body = to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let id = created["id"].as_str().unwrap();

    // Update upstream_url
    let update = json!({ "upstream_url": "http://192.168.1.5:9443" });
    let put_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/applications/{id}"))
                .method("PUT")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie.clone())
                .body(Body::from(serde_json::to_vec(&update).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(put_resp.status(), StatusCode::OK);
    let body = to_bytes(put_resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["upstream_url"], "http://192.168.1.5:9443");
    assert_eq!(updated["name"], "Portainer"); // unchanged

    // Verify via GET
    let get_resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/applications/{id}"))
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_resp.status(), StatusCode::OK);
    let body = to_bytes(get_resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["upstream_url"], "http://192.168.1.5:9443");
}

#[tokio::test]
#[file_serial]
async fn test_duplicate_hostname_returns_409() {
    let (app, _pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;

    let payload = json!({
        "name": "App A",
        "upstream_url": "http://192.168.1.1:80",
        "hostname": "shared.example.com"
    });

    // First create succeeds
    let r1 = app.clone().oneshot(
        Request::builder()
            .uri("/api/applications")
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, cookie.clone())
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(r1.status(), StatusCode::CREATED);

    // Second create with same hostname must fail with 409
    let payload2 = json!({
        "name": "App B",
        "upstream_url": "http://192.168.1.2:80",
        "hostname": "shared.example.com"
    });
    let r2 = app
        .oneshot(
            Request::builder()
                .uri("/api/applications")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie)
                .body(Body::from(serde_json::to_vec(&payload2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r2.status(), StatusCode::CONFLICT);
}
