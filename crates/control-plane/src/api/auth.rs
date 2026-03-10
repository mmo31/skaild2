use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use uuid::Uuid;

use crate::{api::setup::AppError, state::AppState};

pub(crate) const ADMIN_SESSION_KEY: &str = "admin_id";

/// Extract and validate the session admin_id, returning Unauthorized if absent.
pub async fn require_auth(session: &Session) -> Result<Uuid, AppError> {
    session
        .get::<Uuid>(ADMIN_SESSION_KEY)
        .await
        .map_err(|e| AppError::SessionError(e.to_string()))?
        .ok_or(AppError::Unauthorized)
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub success: bool,
}

/// Login endpoint - authenticate and create session
pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Authenticate admin
    let admin = shared::authenticate_admin(&state.db_pool, &payload.email, &payload.password)
        .await?;
    
    // Store admin ID in session
    session
        .insert(ADMIN_SESSION_KEY, admin.id)
        .await
        .map_err(|e| AppError::SessionError(e.to_string()))?;
    
    Ok(Json(AuthResponse {
        id: admin.id.to_string(),
        name: admin.name,
        email: admin.email,
    }))
}

/// Logout endpoint - destroy session
pub async fn logout(session: Session) -> Result<Json<LogoutResponse>, AppError> {
    session
        .delete()
        .await
        .map_err(|e| AppError::SessionError(e.to_string()))?;
    
    Ok(Json(LogoutResponse { success: true }))
}

/// Get current authenticated admin
pub async fn me(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<AuthResponse>, AppError> {
    let admin_id = require_auth(&session).await?;
    
    // Fetch admin from database
    let admin: shared::Admin = sqlx::query_as(
        "SELECT id, name, email, password_hash, created_at, updated_at FROM admins WHERE id = $1"
    )
    .bind(admin_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::Unauthorized)?;
    
    Ok(Json(AuthResponse {
        id: admin.id.to_string(),
        name: admin.name,
        email: admin.email,
    }))
}
