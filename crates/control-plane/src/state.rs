use shared::db::DbPool;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
}

impl AppState {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }
}
