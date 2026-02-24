use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = shared::config::AppConfig::new(
        std::env::var("SKAILD2_DB_URL")
            .unwrap_or_else(|_| "postgres://skaild2:skaild2@localhost:5432/skaild2".to_string()),
    );

    println!("Control-plane starting with config: {:?}", config);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/info", get(api_info));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Control-plane listening on {}", addr);

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
