use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    api::{auth::require_auth, setup::AppError},
    state::AppState,
};
use shared::{AccessMode, CreateRouteInput, Route, RouteWithUpstream, UpdateRouteInput};

// ── Request / Response types ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateRouteRequest {
    pub host: String,
    pub path_prefix: Option<String>,
    pub access_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRouteRequest {
    pub host: Option<String>,
    pub path_prefix: Option<String>,
    pub access_mode: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct RouteResponse {
    pub id: String,
    pub application_id: String,
    pub host: String,
    pub path_prefix: String,
    pub access_mode: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Route> for RouteResponse {
    fn from(r: Route) -> Self {
        Self {
            id: r.id.to_string(),
            application_id: r.application_id.to_string(),
            host: r.host,
            path_prefix: r.path_prefix,
            access_mode: r.access_mode.as_str().to_string(),
            enabled: r.enabled,
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InternalRouteEntry {
    pub id: String,
    pub host: String,
    pub path_prefix: String,
    pub access_mode: String,
    pub upstream_url: String,
}

impl From<RouteWithUpstream> for InternalRouteEntry {
    fn from(r: RouteWithUpstream) -> Self {
        Self {
            id: r.id.to_string(),
            host: r.host,
            path_prefix: r.path_prefix,
            access_mode: r.access_mode,
            upstream_url: r.upstream_url,
        }
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /api/applications/:app_id/routes — create a route (auth required)
pub async fn create_route(
    session: Session,
    State(state): State<AppState>,
    Path(app_id): Path<Uuid>,
    Json(payload): Json<CreateRouteRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_auth(&session).await?;

    // Verify application exists — returns 404 via ApplicationError::NotFound if absent
    shared::get_application_by_id(&state.db_pool, app_id).await?;

    let access_mode = AccessMode::try_from(
        payload.access_mode.as_deref().unwrap_or("login_required"),
    )?;

    let route = shared::create_route(
        &state.db_pool,
        CreateRouteInput {
            application_id: app_id,
            host: payload.host,
            path_prefix: payload.path_prefix,
            access_mode,
        },
    )
    .await?;

    Ok((StatusCode::CREATED, Json(RouteResponse::from(route))))
}

/// GET /api/applications/:app_id/routes — list routes for application (auth required)
pub async fn list_routes(
    session: Session,
    State(state): State<AppState>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<Vec<RouteResponse>>, AppError> {
    require_auth(&session).await?;

    // Verify application exists
    shared::get_application_by_id(&state.db_pool, app_id).await?;

    let routes = shared::list_routes_by_application(&state.db_pool, app_id).await?;
    Ok(Json(routes.into_iter().map(RouteResponse::from).collect()))
}

/// GET /api/routes/:id — get single route (auth required)
pub async fn get_route(
    session: Session,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RouteResponse>, AppError> {
    require_auth(&session).await?;

    let route = shared::get_route_by_id(&state.db_pool, id).await?;
    Ok(Json(RouteResponse::from(route)))
}

/// PUT /api/routes/:id — update route (auth required)
pub async fn update_route(
    session: Session,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRouteRequest>,
) -> Result<Json<RouteResponse>, AppError> {
    require_auth(&session).await?;

    let access_mode = payload
        .access_mode
        .as_deref()
        .map(AccessMode::try_from)
        .transpose()?;

    let route = shared::update_route(
        &state.db_pool,
        id,
        UpdateRouteInput {
            host: payload.host,
            path_prefix: payload.path_prefix,
            access_mode,
            enabled: payload.enabled,
        },
    )
    .await?;

    Ok(Json(RouteResponse::from(route)))
}

/// GET /api/internal/routes — all enabled routes with upstream URL (no auth — internal gateway use)
pub async fn get_internal_routes(
    State(state): State<AppState>,
) -> Result<Json<Vec<InternalRouteEntry>>, AppError> {
    let routes = shared::list_all_enabled_routes_with_upstream(&state.db_pool).await?;
    Ok(Json(routes.into_iter().map(InternalRouteEntry::from).collect()))
}

// ── Connection test ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AuthCheckResult {
    pub configured: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectionTestResult {
    pub status: String,
    pub http_status: Option<u16>,
    pub latency_ms: Option<u64>,
    pub error_kind: Option<String>,
    pub error_message: Option<String>,
    pub auth_check: Option<AuthCheckResult>,
}

fn classify_error(err: &reqwest::Error) -> (String, String) {
    let msg = err.to_string().to_lowercase();
    if err.is_timeout() {
        ("timeout".to_string(), "Connection timed out after 5 seconds".to_string())
    } else if err.is_connect() {
        if msg.contains("dns") || msg.contains("failed to lookup") || msg.contains("resolve") {
            let host = err.url()
                .map(|u| u.host_str().unwrap_or("unknown").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            ("dns".to_string(), format!("DNS resolution failed for host '{host}'"))
        } else if msg.contains("tls") || msg.contains("certificate") || msg.contains("handshake") || msg.contains("ssl") {
            ("tls".to_string(), "TLS certificate validation failed".to_string())
        } else {
            ("connection".to_string(), "Connection refused or host unreachable".to_string())
        }
    } else {
        ("error".to_string(), format!("Request failed: {}", err))
    }
}

/// POST /api/routes/:id/test — run a connection test for a route (auth required)
pub async fn test_route(
    session: Session,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConnectionTestResult>, AppError> {
    require_auth(&session).await?;

    let route = shared::get_route_with_upstream_by_id(&state.db_pool, id).await?;

    // Build test URL: upstream_url (no trailing slash by convention) + path_prefix (starts with /)
    let test_url = format!("{}{}", route.upstream_url.trim_end_matches('/'), route.path_prefix);

    let start = Instant::now();
    let response = state.http_client.get(&test_url).send().await;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    let auth_check = if route.access_mode == "login_required" {
        Some(AuthCheckResult {
            configured: false,
            message: "No identity provider configured — set one up in Epic 3 to enable auth validation".to_string(),
        })
    } else {
        None
    };

    let result = match response {
        Ok(resp) => ConnectionTestResult {
            status: "ok".to_string(),
            http_status: Some(resp.status().as_u16()),
            latency_ms: Some(elapsed_ms),
            error_kind: None,
            error_message: None,
            auth_check,
        },
        Err(err) => {
            let (kind, message) = classify_error(&err);
            ConnectionTestResult {
                status: "error".to_string(),
                http_status: None,
                latency_ms: Some(elapsed_ms),
                error_kind: Some(kind),
                error_message: Some(message),
                auth_check,
            }
        }
    };

    Ok(Json(result))
}
