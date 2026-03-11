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
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::method;

async fn create_test_app() -> (axum::Router, sqlx::PgPool) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://skaild2:skaild2_dev_password@localhost:5432/skaild2_test".to_string());

    let config = shared::config::AppConfig::new(database_url);
    let pool = shared::db::create_pool(&config).await.expect("Failed to create pool");
    shared::db::run_migrations(&pool).await.expect("Failed to run migrations");

    // Clean in FK-safe order
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

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .no_proxy()  // Bypass corporate proxy — tests hit 127.0.0.1 directly
        .build()
        .expect("Failed to build http client");

    let state = control_plane::state::AppState::new(pool.clone(), http_client);
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600)));

    let app = axum::Router::new()
        .route("/api/auth/login", axum::routing::post(control_plane::api::login))
        .route("/api/applications",
               axum::routing::get(control_plane::api::list_applications)
               .post(control_plane::api::create_application))
        .route("/api/applications/:app_id/routes",
               axum::routing::get(control_plane::api::list_routes)
               .post(control_plane::api::create_route))
        .route("/api/routes/:id/test", axum::routing::post(control_plane::api::test_route))
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

async fn create_test_app_and_route(
    app: &axum::Router,
    cookie: &str,
    upstream_url: &str,
    access_mode: &str,
) -> String {
    // Create application
    let app_payload = json!({
        "name": "Test App",
        "upstream_url": upstream_url,
        "hostname": "test.example.com"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/applications")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie)
                .body(Body::from(serde_json::to_vec(&app_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED, "create application failed");
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let app_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let app_id = app_json["id"].as_str().unwrap();

    // Create route
    let route_payload = json!({
        "host": "test.example.com",
        "path_prefix": "/",
        "access_mode": access_mode
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/applications/{app_id}/routes"))
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie)
                .body(Body::from(serde_json::to_vec(&route_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED, "create route failed");
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let route_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    route_json["id"].as_str().unwrap().to_string()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
#[file_serial]
async fn test_connection_test_requires_auth() {
    let (app, _pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;
    let route_id = create_test_app_and_route(&app, &cookie, "http://127.0.0.1:19991", "public").await;

    // No cookie — expect 401
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/routes/{route_id}/test"))
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[file_serial]
async fn test_connection_test_unreachable() {
    let (app, _pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;
    // Port 19991 — nothing bound here
    let route_id = create_test_app_and_route(&app, &cookie, "http://127.0.0.1:19991", "public").await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/routes/{route_id}/test"))
                .method("POST")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["error_kind"], "connection");
    assert!(json["error_message"].as_str().unwrap().len() > 0);
}

#[tokio::test]
#[file_serial]
async fn test_connection_test_dns_failure() {
    let (app, _pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;
    let route_id = create_test_app_and_route(
        &app, &cookie,
        "http://this-host-absolutely-does-not-exist.invalid:8080",
        "public",
    ).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/routes/{route_id}/test"))
                .method("POST")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "error");
    // DNS failures may be classified as "dns", "connection", or "timeout" depending on the OS resolver
    let kind = json["error_kind"].as_str().unwrap();
    assert!(kind == "dns" || kind == "connection" || kind == "timeout", "unexpected error_kind: {kind}");
}

#[tokio::test]
#[file_serial]
async fn test_connection_test_success() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let (app, _pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;
    let route_id = create_test_app_and_route(&app, &cookie, &mock_server.uri(), "public").await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/routes/{route_id}/test"))
                .method("POST")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["http_status"], 200);
    assert!(json["latency_ms"].as_u64().is_some());
    assert!(json["auth_check"].is_null());
}

#[tokio::test]
#[file_serial]
async fn test_connection_test_login_required_auth_check() {
    let (app, _pool) = create_test_app().await;
    let cookie = login_and_get_cookie(&app).await;
    // Use unreachable upstream — we only care about auth_check field
    let route_id = create_test_app_and_route(
        &app, &cookie,
        "http://127.0.0.1:19991",
        "login_required",
    ).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/routes/{route_id}/test"))
                .method("POST")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // auth_check must be present for login_required routes
    assert!(!json["auth_check"].is_null());
    assert_eq!(json["auth_check"]["configured"], false);
    let msg = json["auth_check"]["message"].as_str().unwrap();
    assert!(msg.len() > 0);
}
