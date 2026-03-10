pub mod config {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct AppConfig {
        pub database_url: String,
    }

    impl AppConfig {
        pub fn new(database_url: impl Into<String>) -> Self {
            Self {
                database_url: database_url.into(),
            }
        }

        pub fn from_env() -> Result<Self, String> {
            std::env::var("DATABASE_URL")
                .map(|url| Self::new(url))
                .map_err(|_| "DATABASE_URL environment variable not set".to_string())
        }
    }
}

pub mod db {
    use sqlx::{PgPool, postgres::PgPoolOptions};
    use super::config::AppConfig;

    /// Real database connection pool using sqlx
    pub type DbPool = PgPool;

    /// Create a new database connection pool
    pub async fn create_pool(config: &AppConfig) -> Result<DbPool, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await
    }

    /// Run database migrations
    pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("../../migrations")
            .run(pool)
            .await
    }
}

pub mod models;

pub use models::*;

#[cfg(test)]
mod tests {
    use super::config::AppConfig;

    #[test]
    fn app_config_stores_database_url() {
        let cfg = AppConfig::new("postgres://user:pass@localhost:5432/db");
        assert_eq!(cfg.database_url, "postgres://user:pass@localhost:5432/db");
    }

    #[test]
    fn app_config_accepts_string_types() {
        let owned = AppConfig::new(String::from("postgres://localhost/test"));
        let borrowed = AppConfig::new("postgres://localhost/test");
        assert_eq!(owned.database_url, borrowed.database_url);
    }

    #[test]
    fn app_config_from_env_fails_when_unset() {
        unsafe { std::env::remove_var("DATABASE_URL") };
        let result = AppConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not set"));
    }

    #[test]
    fn app_config_from_env_succeeds_when_set() {
        unsafe { std::env::set_var("DATABASE_URL", "postgres://test:test@localhost/test") };
        let result = AppConfig::from_env();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().database_url, "postgres://test:test@localhost/test");
        unsafe { std::env::remove_var("DATABASE_URL") };
    }
}
