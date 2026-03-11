use shared::db::DbPool;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(db_pool: DbPool, http_client: reqwest::Client) -> Self {
        Self { db_pool, http_client }
    }
}
