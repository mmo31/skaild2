use shared::{ApplicationError, RouteError};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use shared::{AdminError, CreateAdminInput};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SetupStatusResponse {
    pub setup_complete: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateAdminRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct CreateAdminResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

/// Check if setup is complete (whether any admins exist)
pub async fn get_setup_status(
    State(state): State<AppState>,
) -> Result<Json<SetupStatusResponse>, AppError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admins")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    Ok(Json(SetupStatusResponse {
        setup_complete: count.0 > 0,
    }))
}

/// Alias endpoint (POST) for setup status, matching Story 1.1 task naming.
pub async fn setup_init(
    State(state): State<AppState>,
) -> Result<Json<SetupStatusResponse>, AppError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admins")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(SetupStatusResponse {
        setup_complete: count.0 > 0,
    }))
}

/// Create the first admin account
pub async fn create_first_admin(
    State(state): State<AppState>,
    Json(payload): Json<CreateAdminRequest>,
) -> Result<Json<CreateAdminResponse>, AppError> {
    // Check if setup already complete
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admins")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    if count.0 > 0 {
        return Err(AppError::SetupAlreadyComplete);
    }
    
    // Create admin
    let admin = shared::create_admin(
        &state.db_pool,
        CreateAdminInput {
            name: payload.name,
            email: payload.email,
            password: payload.password,
        },
    )
    .await?;
    
    Ok(Json(CreateAdminResponse {
        id: admin.id.to_string(),
        name: admin.name,
        email: admin.email,
        created_at: admin.created_at.to_rfc3339(),
    }))
}

/// Application error type
#[derive(Debug)]
pub enum AppError {
    AdminError(AdminError),
    ApplicationError(ApplicationError),
    RouteError(RouteError),
    DatabaseError(String),
    SetupAlreadyComplete,
    SessionError(String),
    Unauthorized,
}

impl From<AdminError> for AppError {
    fn from(err: AdminError) -> Self {
        AppError::AdminError(err)
    }
}

impl From<ApplicationError> for AppError {
    fn from(err: ApplicationError) -> Self {
        AppError::ApplicationError(err)
    }
}

impl From<RouteError> for AppError {
    fn from(err: RouteError) -> Self {
        AppError::RouteError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::AdminError(AdminError::InvalidEmail) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Invalid email address")
            }
            AppError::AdminError(AdminError::PasswordTooShort) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Password must be at least 12 characters")
            }
            AppError::AdminError(AdminError::PasswordNeedsUppercase) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Password must contain uppercase letter")
            }
            AppError::AdminError(AdminError::PasswordNeedsLowercase) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Password must contain lowercase letter")
            }
            AppError::AdminError(AdminError::PasswordNeedsNumber) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Password must contain a number")
            }
            AppError::AdminError(AdminError::PasswordNeedsSpecial) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Password must contain special character")
            }
            AppError::AdminError(AdminError::InvalidCredentials) => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials")
            }
            AppError::AdminError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::ApplicationError(ApplicationError::NameRequired) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Application name is required")
            }
            AppError::ApplicationError(ApplicationError::InvalidUpstreamUrl(_)) => {
                (StatusCode::BAD_REQUEST, "Upstream URL is invalid")
            }
            AppError::ApplicationError(ApplicationError::HostnameRequired) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Hostname is required")
            }
            AppError::ApplicationError(ApplicationError::HostnameConflict) => {
                (StatusCode::CONFLICT, "Hostname is already in use")
            }
            AppError::ApplicationError(ApplicationError::NotFound) => {
                (StatusCode::NOT_FOUND, "Application not found")
            }
            AppError::ApplicationError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::RouteError(RouteError::HostRequired) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Host is required")
            }
            AppError::RouteError(RouteError::ApplicationNotFound) => {
                (StatusCode::NOT_FOUND, "Application not found")
            }
            AppError::RouteError(RouteError::NotFound) => {
                (StatusCode::NOT_FOUND, "Route not found")
            }
            AppError::RouteError(RouteError::DuplicateHostPath) => {
                (StatusCode::CONFLICT, "Route with this host and path already exists")
            }
            AppError::RouteError(RouteError::InvalidAccessMode(_)) => {
                (StatusCode::BAD_REQUEST, "Invalid access mode")
            }
            AppError::RouteError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            AppError::SetupAlreadyComplete => {
                (StatusCode::BAD_REQUEST, "Setup already complete")
            }
            AppError::SessionError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Session error")
            }
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized")
            }
        };

        let body = Json(serde_json::json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
