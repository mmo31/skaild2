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
            std::env::var("SKAILD2_DB_URL")
                .map(|url| Self::new(url))
                .map_err(|_| "SKAILD2_DB_URL environment variable not set".to_string())
        }
    }
}

pub mod db {
    use super::config::AppConfig;

    /// DbPool placeholder for shared database configuration.
    /// Note: This is a configuration wrapper, not an actual connection pool yet.
    /// Future stories will integrate a real pool library (e.g., sqlx, deadpool).
    #[derive(Debug)]
    pub struct DbPool {
        pub config: AppConfig,
    }

    impl DbPool {
        pub fn new(config: AppConfig) -> Self {
            Self { config }
        }

        pub fn database_url(&self) -> &str {
            &self.config.database_url
        }
    }
}

#[cfg(test)]
mod tests {
    use super::config::AppConfig;
    use super::db::DbPool;

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
        unsafe { std::env::remove_var("SKAILD2_DB_URL") };
        let result = AppConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not set"));
    }

    #[test]
    fn app_config_from_env_succeeds_when_set() {
        unsafe { std::env::set_var("SKAILD2_DB_URL", "postgres://test:test@localhost/test") };
        let result = AppConfig::from_env();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().database_url, "postgres://test:test@localhost/test");
        unsafe { std::env::remove_var("SKAILD2_DB_URL") };
    }

    #[test]
    fn db_pool_wraps_config() {
        let cfg = AppConfig::new("postgres://user:pass@localhost:5432/db");
        let pool = DbPool::new(cfg.clone());
        assert_eq!(pool.config, cfg);
    }

    #[test]
    fn db_pool_provides_database_url() {
        let cfg = AppConfig::new("postgres://user:pass@localhost:5432/db");
        let pool = DbPool::new(cfg);
        assert_eq!(pool.database_url(), "postgres://user:pass@localhost:5432/db");
    }
}
