use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Access mode for a route
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessMode {
    Public,
    LoginRequired,
}

impl AccessMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessMode::Public => "public",
            AccessMode::LoginRequired => "login_required",
        }
    }
}

impl TryFrom<&str> for AccessMode {
    type Error = RouteError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "public" => Ok(AccessMode::Public),
            "login_required" => Ok(AccessMode::LoginRequired),
            _ => Err(RouteError::InvalidAccessMode(s.to_string())),
        }
    }
}

/// Raw DB row — access_mode stored as String to avoid PG ENUM / sqlx runtime complications
#[derive(Debug, Clone, FromRow)]
pub struct RouteRow {
    pub id: Uuid,
    pub application_id: Uuid,
    pub host: String,
    pub path_prefix: String,
    pub access_mode: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Route as returned by the API — access_mode as typed enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub id: Uuid,
    pub application_id: Uuid,
    pub host: String,
    pub path_prefix: String,
    pub access_mode: AccessMode,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<RouteRow> for Route {
    type Error = RouteError;
    fn try_from(r: RouteRow) -> Result<Self, Self::Error> {
        let access_mode = AccessMode::try_from(r.access_mode.as_str())?;
        Ok(Route {
            id: r.id,
            application_id: r.application_id,
            host: r.host,
            path_prefix: r.path_prefix,
            access_mode,
            enabled: r.enabled,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

/// Route joined with its application's upstream_url — used by the gateway config endpoint
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RouteWithUpstream {
    pub id: Uuid,
    pub host: String,
    pub path_prefix: String,
    pub access_mode: String,
    pub upstream_url: String,
}

/// Input for creating a new route
#[derive(Debug, Clone)]
pub struct CreateRouteInput {
    pub application_id: Uuid,
    pub host: String,
    pub path_prefix: Option<String>,
    pub access_mode: AccessMode,
}

/// Input for updating a route — all fields optional
#[derive(Debug, Clone)]
pub struct UpdateRouteInput {
    pub host: Option<String>,
    pub path_prefix: Option<String>,
    pub access_mode: Option<AccessMode>,
    pub enabled: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum RouteError {
    #[error("Host is required")]
    HostRequired,
    #[error("Application not found")]
    ApplicationNotFound,
    #[error("Route not found")]
    NotFound,
    #[error("Route with this host and path already exists for this application")]
    DuplicateHostPath,
    #[error("Invalid access mode: {0}")]
    InvalidAccessMode(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Create a new route
pub async fn create_route(
    pool: &crate::db::DbPool,
    input: CreateRouteInput,
) -> Result<Route, RouteError> {
    let host = input.host.trim().to_string();
    if host.is_empty() {
        return Err(RouteError::HostRequired);
    }
    let path_prefix = input.path_prefix.unwrap_or_else(|| "/".to_string());

    let row = sqlx::query_as::<_, RouteRow>(
        r#"
        INSERT INTO routes (application_id, host, path_prefix, access_mode)
        VALUES ($1, $2, $3, $4)
        RETURNING id, application_id, host, path_prefix, access_mode, enabled, created_at, updated_at
        "#,
    )
    .bind(input.application_id)
    .bind(&host)
    .bind(&path_prefix)
    .bind(input.access_mode.as_str())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("idx_routes_host_path") || msg.contains("unique") {
            RouteError::DuplicateHostPath
        } else if msg.contains("foreign key") || msg.contains("violates") {
            RouteError::ApplicationNotFound
        } else {
            RouteError::DatabaseError(msg)
        }
    })?;

    Route::try_from(row)
}

/// List all routes for an application ordered by creation date ascending
pub async fn list_routes_by_application(
    pool: &crate::db::DbPool,
    application_id: Uuid,
) -> Result<Vec<Route>, RouteError> {
    let rows = sqlx::query_as::<_, RouteRow>(
        "SELECT id, application_id, host, path_prefix, access_mode, enabled, created_at, updated_at \
         FROM routes WHERE application_id = $1 ORDER BY created_at ASC",
    )
    .bind(application_id)
    .fetch_all(pool)
    .await
    .map_err(|e| RouteError::DatabaseError(e.to_string()))?;

    rows.into_iter().map(Route::try_from).collect()
}

/// Get a single route by id
pub async fn get_route_by_id(
    pool: &crate::db::DbPool,
    id: Uuid,
) -> Result<Route, RouteError> {
    let row = sqlx::query_as::<_, RouteRow>(
        "SELECT id, application_id, host, path_prefix, access_mode, enabled, created_at, updated_at \
         FROM routes WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| RouteError::DatabaseError(e.to_string()))?
    .ok_or(RouteError::NotFound)?;

    Route::try_from(row)
}

/// Update mutable fields of a route; fetch current row to preserve unchanged fields
pub async fn update_route(
    pool: &crate::db::DbPool,
    id: Uuid,
    input: UpdateRouteInput,
) -> Result<Route, RouteError> {
    // Validate new host if provided
    if let Some(ref h) = input.host {
        if h.trim().is_empty() {
            return Err(RouteError::HostRequired);
        }
    }

    // Fetch current row to fill unchanged fields
    let current = sqlx::query_as::<_, RouteRow>(
        "SELECT id, application_id, host, path_prefix, access_mode, enabled, created_at, updated_at \
         FROM routes WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| RouteError::DatabaseError(e.to_string()))?
    .ok_or(RouteError::NotFound)?;

    let new_host = input.host.unwrap_or(current.host);
    let new_path_prefix = input.path_prefix.unwrap_or(current.path_prefix);
    let new_access_mode = input
        .access_mode
        .map(|m| m.as_str().to_string())
        .unwrap_or(current.access_mode);
    let new_enabled = input.enabled.unwrap_or(current.enabled);

    let row = sqlx::query_as::<_, RouteRow>(
        r#"
        UPDATE routes
        SET host = $1, path_prefix = $2, access_mode = $3, enabled = $4, updated_at = NOW()
        WHERE id = $5
        RETURNING id, application_id, host, path_prefix, access_mode, enabled, created_at, updated_at
        "#,
    )
    .bind(&new_host)
    .bind(&new_path_prefix)
    .bind(&new_access_mode)
    .bind(new_enabled)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("idx_routes_host_path") || msg.contains("unique") {
            RouteError::DuplicateHostPath
        } else {
            RouteError::DatabaseError(msg)
        }
    })?
    .ok_or(RouteError::NotFound)?;

    Route::try_from(row)
}

/// List all enabled routes joined with their application's upstream_url, sorted for longest-prefix match
pub async fn list_all_enabled_routes_with_upstream(
    pool: &crate::db::DbPool,
) -> Result<Vec<RouteWithUpstream>, RouteError> {
    sqlx::query_as::<_, RouteWithUpstream>(
        r#"
        SELECT r.id, r.host, r.path_prefix, r.access_mode, a.upstream_url
        FROM routes r
        JOIN applications a ON a.id = r.application_id
        WHERE r.enabled = true AND a.enabled = true
        ORDER BY LENGTH(r.path_prefix) DESC, r.host ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| RouteError::DatabaseError(e.to_string()))
}

/// Get a single route with its application's upstream_url by route id.
/// No `enabled` filter — admins can test disabled routes.
pub async fn get_route_with_upstream_by_id(
    pool: &crate::db::DbPool,
    id: Uuid,
) -> Result<RouteWithUpstream, RouteError> {
    sqlx::query_as::<_, RouteWithUpstream>(
        r#"
        SELECT r.id, r.host, r.path_prefix, r.access_mode, a.upstream_url
        FROM routes r
        JOIN applications a ON a.id = r.application_id
        WHERE r.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| RouteError::DatabaseError(e.to_string()))?
    .ok_or(RouteError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_mode_parse_public() {
        assert_eq!(AccessMode::try_from("public").unwrap(), AccessMode::Public);
    }

    #[test]
    fn test_access_mode_parse_login_required() {
        assert_eq!(
            AccessMode::try_from("login_required").unwrap(),
            AccessMode::LoginRequired
        );
    }

    #[test]
    fn test_access_mode_parse_invalid() {
        assert!(AccessMode::try_from("admin_only").is_err());
    }

    #[test]
    fn test_access_mode_parse_empty() {
        assert!(AccessMode::try_from("").is_err());
    }

    #[test]
    fn test_access_mode_as_str_public() {
        assert_eq!(AccessMode::Public.as_str(), "public");
    }

    #[test]
    fn test_access_mode_as_str_login_required() {
        assert_eq!(AccessMode::LoginRequired.as_str(), "login_required");
    }

    #[test]
    fn test_route_try_from_row_valid_public() {
        let row = RouteRow {
            id: Uuid::nil(),
            application_id: Uuid::nil(),
            host: "app.example.com".to_string(),
            path_prefix: "/".to_string(),
            access_mode: "public".to_string(),
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let route = Route::try_from(row).unwrap();
        assert_eq!(route.access_mode, AccessMode::Public);
    }

    #[test]
    fn test_route_try_from_row_valid_login_required() {
        let row = RouteRow {
            id: Uuid::nil(),
            application_id: Uuid::nil(),
            host: "app.example.com".to_string(),
            path_prefix: "/api".to_string(),
            access_mode: "login_required".to_string(),
            enabled: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let route = Route::try_from(row).unwrap();
        assert_eq!(route.access_mode, AccessMode::LoginRequired);
        assert!(!route.enabled);
    }

    #[test]
    fn test_route_try_from_row_invalid_access_mode() {
        let row = RouteRow {
            id: Uuid::nil(),
            application_id: Uuid::nil(),
            host: "app.example.com".to_string(),
            path_prefix: "/".to_string(),
            access_mode: "garbage".to_string(),
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(Route::try_from(row).is_err());
    }
}
