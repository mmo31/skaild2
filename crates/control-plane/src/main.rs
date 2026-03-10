use control_plane::{api, state::AppState};
use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{RedisStore, fred::prelude::*};
use time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env if present
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load database configuration
    let config = shared::config::AppConfig::from_env()
        .unwrap_or_else(|_| {
            tracing::warn!("DATABASE_URL not set, using default");
            shared::config::AppConfig::new("postgres://skaild2:skaild2_dev_password@postgres:5432/skaild2")
        });

    tracing::info!("Control-plane starting with database: {}", config.database_url);

    // Create database connection pool
    let pool = shared::db::create_pool(&config).await?;
    tracing::info!("Database connection pool created");

    // Run migrations
    shared::db::run_migrations(&pool).await?;
    tracing::info!("Database migrations complete");

    // Set up Redis for session storage
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://redis:6379".to_string());
    tracing::info!("Connecting to Redis: {}", redis_url);

    let redis_config = RedisConfig::from_url(&redis_url)?;
    let redis_client = RedisClient::new(redis_config, None, None, None);
    redis_client.connect();
    redis_client.wait_for_connect().await?;

    let session_store = RedisStore::new(redis_client);

    let cookie_secure = std::env::var("COOKIE_SECURE")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false);

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(cookie_secure)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600))); // 1 hour

    tracing::info!("Session management configured");

    // Create application state
    let app_state = AppState::new(pool);

    // Configure CORS - allow admin UI origin
    let frontend_origin = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());

    let frontend_origin_header = frontend_origin
        .parse::<axum::http::HeaderValue>()
        .unwrap_or_else(|err| {
            tracing::warn!(
                "Invalid FRONTEND_URL for CORS ({}): {}; falling back to http://localhost:5173",
                frontend_origin,
                err
            );
            axum::http::HeaderValue::from_static("http://localhost:5173")
        });
    
    let cors = CorsLayer::new()
        .allow_origin(frontend_origin_header)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(true);

    // Build application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/info", get(api_info))
        // Setup endpoints
        .route("/api/setup/status", get(api::get_setup_status))
        .route("/api/setup/init", post(api::setup_init))
        .route("/api/setup/create-admin", post(api::create_first_admin))
        // Auth endpoints
        .route("/api/auth/login", post(api::login))
        .route("/api/auth/logout", post(api::logout))
        .route("/api/auth/me", get(api::me))
        // Application endpoints
        .route("/api/applications", get(api::list_applications).post(api::create_application))
        .route("/api/applications/:id", get(api::get_application).put(api::update_application))
        .layer(cors)
        .layer(session_layer)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Control-plane listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn api_info() -> &'static str {
    "skaild2 control-plane API"
}
