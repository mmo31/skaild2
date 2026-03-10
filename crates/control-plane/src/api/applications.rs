use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use uuid::Uuid;

use crate::{api::{auth::require_auth, setup::AppError}, state::AppState};
use shared::{Application, CreateApplicationInput, UpdateApplicationInput};

// ── Request / Response types ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateApplicationRequest {
    pub name: String,
    pub upstream_url: String,
    pub hostname: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateApplicationRequest {
    pub name: Option<String>,
    pub upstream_url: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApplicationResponse {
    pub id: String,
    pub name: String,
    pub upstream_url: String,
    pub hostname: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Application> for ApplicationResponse {
    fn from(a: Application) -> Self {
        Self {
            id: a.id.to_string(),
            name: a.name,
            upstream_url: a.upstream_url,
            hostname: a.hostname,
            enabled: a.enabled,
            created_at: a.created_at.to_rfc3339(),
            updated_at: a.updated_at.to_rfc3339(),
        }
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /api/applications — create a new application (auth required)
pub async fn create_application(
    session: Session,
    State(state): State<AppState>,
    Json(payload): Json<CreateApplicationRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_auth(&session).await?;

    let app = shared::create_application(
        &state.db_pool,
        CreateApplicationInput {
            name: payload.name,
            upstream_url: payload.upstream_url,
            hostname: payload.hostname,
        },
    )
    .await?;

    Ok((StatusCode::CREATED, Json(ApplicationResponse::from(app))))
}

/// GET /api/applications — list all applications (auth required)
pub async fn list_applications(
    session: Session,
    State(state): State<AppState>,
) -> Result<Json<Vec<ApplicationResponse>>, AppError> {
    require_auth(&session).await?;

    let apps = shared::list_applications(&state.db_pool).await?;
    Ok(Json(apps.into_iter().map(ApplicationResponse::from).collect()))
}

/// GET /api/applications/:id — get a single application (auth required)
pub async fn get_application(
    session: Session,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApplicationResponse>, AppError> {
    require_auth(&session).await?;

    let app = shared::get_application_by_id(&state.db_pool, id).await?;
    Ok(Json(ApplicationResponse::from(app)))
}

/// PUT /api/applications/:id — update application fields (auth required)
pub async fn update_application(
    session: Session,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateApplicationRequest>,
) -> Result<Json<ApplicationResponse>, AppError> {
    require_auth(&session).await?;

    let app = shared::update_application(
        &state.db_pool,
        id,
        UpdateApplicationInput {
            name: payload.name,
            upstream_url: payload.upstream_url,
            hostname: payload.hostname,
        },
    )
    .await?;

    Ok(Json(ApplicationResponse::from(app)))
}
