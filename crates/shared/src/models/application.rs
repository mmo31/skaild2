use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Upstream application registered for proxying
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Application {
    pub id: Uuid,
    pub name: String,
    pub upstream_url: String,
    pub hostname: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new application
#[derive(Debug, Clone)]
pub struct CreateApplicationInput {
    pub name: String,
    pub upstream_url: String,
    pub hostname: String,
}

/// Input for updating an existing application (all fields optional)
#[derive(Debug, Clone)]
pub struct UpdateApplicationInput {
    pub name: Option<String>,
    pub upstream_url: Option<String>,
    pub hostname: Option<String>,
}

/// Validation and persistence errors for application operations
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Application name is required")]
    NameRequired,
    #[error("Upstream URL is invalid: {0}")]
    InvalidUpstreamUrl(String),
    #[error("Hostname is required")]
    HostnameRequired,
    #[error("Hostname is already in use")]
    HostnameConflict,
    #[error("Application not found")]
    NotFound,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Validate that an upstream URL is a valid http/https URL with a host
fn validate_upstream_url(raw: &str) -> Result<(), ApplicationError> {
    let parsed = url::Url::parse(raw)
        .map_err(|e| ApplicationError::InvalidUpstreamUrl(e.to_string()))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(ApplicationError::InvalidUpstreamUrl(
            "scheme must be http or https".to_string(),
        ));
    }
    if parsed.host_str().map(|h| h.is_empty()).unwrap_or(true) {
        return Err(ApplicationError::InvalidUpstreamUrl(
            "URL must include a host".to_string(),
        ));
    }
    Ok(())
}

/// Create a new application in the database
pub async fn create_application(
    pool: &crate::db::DbPool,
    input: CreateApplicationInput,
) -> Result<Application, ApplicationError> {
    if input.name.trim().is_empty() {
        return Err(ApplicationError::NameRequired);
    }
    validate_upstream_url(&input.upstream_url)?;
    if input.hostname.trim().is_empty() {
        return Err(ApplicationError::HostnameRequired);
    }

    sqlx::query_as::<_, Application>(
        r#"
        INSERT INTO applications (name, upstream_url, hostname)
        VALUES ($1, $2, $3)
        RETURNING id, name, upstream_url, hostname, enabled, created_at, updated_at
        "#,
    )
    .bind(&input.name)
    .bind(&input.upstream_url)
    .bind(&input.hostname)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("idx_applications_hostname") || msg.contains("unique") {
            ApplicationError::HostnameConflict
        } else {
            ApplicationError::DatabaseError(msg)
        }
    })
}

/// List all applications ordered by creation date ascending
pub async fn list_applications(
    pool: &crate::db::DbPool,
) -> Result<Vec<Application>, ApplicationError> {
    sqlx::query_as::<_, Application>(
        "SELECT id, name, upstream_url, hostname, enabled, created_at, updated_at FROM applications ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ApplicationError::DatabaseError(e.to_string()))
}

/// Get a single application by its id
pub async fn get_application_by_id(
    pool: &crate::db::DbPool,
    id: Uuid,
) -> Result<Application, ApplicationError> {
    sqlx::query_as::<_, Application>(
        "SELECT id, name, upstream_url, hostname, enabled, created_at, updated_at FROM applications WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?
    .ok_or(ApplicationError::NotFound)
}

/// Update mutable fields of an application; at least one field must be provided
pub async fn update_application(
    pool: &crate::db::DbPool,
    id: Uuid,
    input: UpdateApplicationInput,
) -> Result<Application, ApplicationError> {
    // Validate any provided values before hitting the DB
    if let Some(ref u) = input.upstream_url {
        validate_upstream_url(u)?;
    }
    if let Some(ref n) = input.name {
        if n.trim().is_empty() {
            return Err(ApplicationError::NameRequired);
        }
    }
    if let Some(ref h) = input.hostname {
        if h.trim().is_empty() {
            return Err(ApplicationError::HostnameRequired);
        }
    }

    // Fetch current row so we can fill in unchanged fields
    let current = get_application_by_id(pool, id).await?;

    let new_name = input.name.unwrap_or(current.name);
    let new_url = input.upstream_url.unwrap_or(current.upstream_url);
    let new_hostname = input.hostname.unwrap_or(current.hostname);

    sqlx::query_as::<_, Application>(
        r#"
        UPDATE applications
        SET name = $1, upstream_url = $2, hostname = $3, updated_at = NOW()
        WHERE id = $4
        RETURNING id, name, upstream_url, hostname, enabled, created_at, updated_at
        "#,
    )
    .bind(&new_name)
    .bind(&new_url)
    .bind(&new_hostname)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("idx_applications_hostname") || msg.contains("unique") {
            ApplicationError::HostnameConflict
        } else {
            ApplicationError::DatabaseError(msg)
        }
    })?
    .ok_or(ApplicationError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_upstream_url_http() {
        assert!(validate_upstream_url("http://192.168.1.10:8080").is_ok());
    }

    #[test]
    fn test_valid_upstream_url_https() {
        assert!(validate_upstream_url("https://192.168.1.10:443").is_ok());
    }

    #[test]
    fn test_invalid_upstream_url_ftp_scheme() {
        let err = validate_upstream_url("ftp://server").unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidUpstreamUrl(_)));
    }

    #[test]
    fn test_invalid_upstream_url_no_scheme() {
        let err = validate_upstream_url("not-a-url").unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidUpstreamUrl(_)));
    }

    #[test]
    fn test_invalid_upstream_url_empty() {
        let err = validate_upstream_url("").unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidUpstreamUrl(_)));
    }

    #[test]
    fn test_validate_upstream_url_no_host() {
        // http:// with no host is rejected by the url crate itself
        let err = validate_upstream_url("http://").unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidUpstreamUrl(_)));
    }
}
