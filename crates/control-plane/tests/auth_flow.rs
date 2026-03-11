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

// Helper to create test app
async fn create_test_app() -> (axum::Router, sqlx::PgPool) {
    // Set up test database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://skaild2:skaild2_dev_password@localhost:5432/skaild2_test".to_string());
    
    let config = shared::config::AppConfig::new(database_url);
    let pool = shared::db::create_pool(&config).await.expect("Failed to create pool");
    
    // Run migrations
    shared::db::run_migrations(&pool).await.expect("Failed to run migrations");
    
    // Clean up any existing admins
    sqlx::query("DELETE FROM admins")
        .execute(&pool)
        .await
        .expect("Failed to clean admins");
    
    // Create app state
    let state = control_plane::state::AppState::new(pool.clone(), reqwest::Client::new());

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600)));
    
    // Create router
    let app = axum::Router::new()
        .route("/api/setup/status", axum::routing::get(control_plane::api::get_setup_status))
        .route("/api/setup/init", axum::routing::post(control_plane::api::setup_init))
        .route("/api/setup/create-admin", axum::routing::post(control_plane::api::create_first_admin))
        .route("/api/auth/login", axum::routing::post(control_plane::api::login))
        .route("/api/auth/logout", axum::routing::post(control_plane::api::logout))
        .route("/api/auth/me", axum::routing::get(control_plane::api::me))
        .layer(session_layer)
        .with_state(state);
    
    (app, pool)
}

#[tokio::test]
#[file_serial]
async fn test_setup_status_no_admins() {
    let (app, _pool) = create_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/setup/status")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["setup_complete"], false);
}

#[tokio::test]
#[file_serial]
async fn test_create_first_admin_success() {
    let (app, pool) = create_test_app().await;
    
    let payload = json!({
        "name": "Test Admin",
        "email": "admin@example.com",
        "password": "SecurePassword123!"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/setup/create-admin")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["email"], "admin@example.com");
    assert_eq!(json["name"], "Test Admin");
    assert!(json["id"].is_string());
    
    // Verify admin was created in database
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admins")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count.0, 1);
}

#[tokio::test]
#[file_serial]
async fn test_me_requires_authentication() {
    let (app, _pool) = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[file_serial]
async fn test_login_and_me_and_logout_flow() {
    let (app, _pool) = create_test_app().await;

    // Create admin
    let create_payload = json!({
        "name": "Test Admin",
        "email": "admin@example.com",
        "password": "SecurePassword123!"
    });

    let _create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/setup/create-admin")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Login
    let login_payload = json!({
        "email": "admin@example.com",
        "password": "SecurePassword123!"
    });

    let login_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_resp.status(), StatusCode::OK);

    let set_cookie = login_resp
        .headers()
        .get(header::SET_COOKIE)
        .expect("expected Set-Cookie header")
        .to_str()
        .unwrap();

    let cookie_kv = set_cookie
        .split(';')
        .next()
        .expect("expected cookie key/value")
        .to_string();

    // /me should work with cookie
    let me_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/me")
                .header(header::COOKIE, cookie_kv.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(me_resp.status(), StatusCode::OK);

    // Logout
    let logout_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/logout")
                .method("POST")
                .header(header::COOKIE, cookie_kv.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(logout_resp.status(), StatusCode::OK);

    // /me should now fail
    let me_after_logout_resp = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/me")
                .header(header::COOKIE, cookie_kv)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(me_after_logout_resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[file_serial]
async fn test_login_with_invalid_credentials_fails() {
    let (app, _pool) = create_test_app().await;

    // Create admin
    let create_payload = json!({
        "name": "Test Admin",
        "email": "admin@example.com",
        "password": "SecurePassword123!"
    });

    let _create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/setup/create-admin")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Wrong password
    let login_payload = json!({
        "email": "admin@example.com",
        "password": "WrongPassword123!"
    });

    let login_resp = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[file_serial]
async fn test_create_admin_with_weak_password() {
    let (app, _pool) = create_test_app().await;
    
    let payload = json!({
        "name": "Test Admin",
        "email": "admin@example.com",
        "password": "weak"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/setup/create-admin")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["error"].as_str().unwrap().contains("12 characters"));
}

#[tokio::test]
#[file_serial]
async fn test_create_admin_twice_fails() {
    let (app, _pool) = create_test_app().await;
    
    let payload = json!({
        "name": "Test Admin",
        "email": "admin@example.com",
        "password": "SecurePassword123!"
    });
    
    // Create first admin
    let _response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/setup/create-admin")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    // Try to create second admin - should fail
    let response2 = app
        .oneshot(
            Request::builder()
                .uri("/api/setup/create-admin")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response2.status(), StatusCode::BAD_REQUEST);
    
    let body = to_bytes(response2.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["error"].as_str().unwrap().contains("already complete"));
}

#[tokio::test]
#[file_serial]
async fn test_setup_status_with_admin() {
    let (app, _pool) = create_test_app().await;
    
    // Create an admin first
    let payload = json!({
        "name": "Test Admin",
        "email": "admin@example.com",
        "password": "SecurePassword123!"
    });
    
    let _response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/setup/create-admin")
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();
    
    // Check setup status
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/setup/status")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["setup_complete"], true);
}
