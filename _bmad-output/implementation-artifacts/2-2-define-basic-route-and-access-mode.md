# Story 2.2: Define Basic Route and Access Mode

Status: done

## Story

As a homelab admin,
I want to define at least one route for an application and choose whether it requires login or is public,
so that I can control how users reach the upstream service through skaild2.

## Acceptance Criteria

1. **Given** at least one application is registered, **when** I open that application's detail view and choose to add a route, **then** I can specify at minimum a host (pre-filled from the application's hostname), an optional path prefix, and an access mode (public vs login-required).

2. **Given** I create a login-required route for the application, **when** I save the route configuration, **then** the route is persisted and appears in the application's route list with its access mode clearly indicated.

3. **Given** the route exists and the gateway is running, **when** I browse to the route's external URL in a browser as an unauthenticated user, **then** the gateway returns a 401 Unauthorized response (IdP redirect is deferred to Epic 3 when IdP is configured).

4. **Given** a `login_required` route exists and the gateway is loading routes from the control-plane, **when** the route configuration is saved, **then** the gateway picks it up within at most 60 seconds and enforces the access mode.

5. **Given** a public route exists for an application, **when** I browse to that route's external URL, **then** I can reach the upstream service without being forced through the identity provider login flow.

> **Scope Note (AC3/AC4):** Full OIDC redirect to an identity provider login flow requires IdP configuration (Epic 3). In this story the gateway enforces `login_required` by returning `401 Unauthorized`. The redirect to an IdP login page is implemented when Epic 3 IdP stories are complete.

## Tasks / Subtasks

- [x] Add database migration for routes table (AC: 2, 3, 5)
  - [x] Create `migrations/20260310000001_create_routes.sql`
  - [x] Define `routes` table: id, application_id (FK), host, path_prefix, access_mode, enabled, created_at, updated_at
  - [x] Use `VARCHAR(50) + CHECK` for access_mode (simpler than PG ENUM with SQLx runtime mode)

- [x] Implement Route model and persistence in shared crate (AC: 2, 3, 4, 5)
  - [x] Create `crates/shared/src/models/route.rs`
  - [x] Define `Route` struct (sqlx `FromRow`, Serialize, Deserialize)
  - [x] Define `AccessMode` enum with `Serialize/Deserialize` (public | login_required)
  - [x] Add `CreateRouteInput` and `UpdateRouteInput` structs
  - [x] Implement `create_route()`, `list_routes_by_application()`, `get_route_by_id()`, `update_route()` async fns
  - [x] Add `RouteError` enum (thiserror)
  - [x] Register `route` module in `shared/src/models/mod.rs`
  - [x] Add unit tests for validation logic

- [x] Build control-plane API endpoints (AC: 1, 2, 3, 4, 5)
  - [x] Create `crates/control-plane/src/api/routes.rs`
  - [x] `POST /api/applications/:app_id/routes` — create route (auth required)
  - [x] `GET /api/applications/:app_id/routes` — list routes for application (auth required)
  - [x] `GET /api/routes/:id` — get single route (auth required)
  - [x] `PUT /api/routes/:id` — update route (auth required)
  - [x] `GET /api/internal/routes` — list all enabled routes with upstream_url (no auth — internal only)
  - [x] Add `RouteError` variants to `AppError` in `setup.rs`
  - [x] Export handlers from `api/mod.rs`
  - [x] Register all routes in `main.rs`

- [x] Extend admin UI (AC: 1, 2)
  - [x] Add Route types and CRUD functions to `admin-ui/src/services/api.ts`
  - [x] Create `admin-ui/src/components/RouteForm.tsx` — shared form for create/edit
  - [x] Extend `admin-ui/src/pages/ApplicationDetailPage.tsx` — add Routes section below application details
  - [x] Show route list (host, path_prefix, access_mode, enabled) with "Add Route" button
  - [x] Empty state: "No routes yet. Add your first one."

- [x] Implement basic gateway routing and enforcement (AC: 3, 4, 5)
  - [x] Add `reqwest`, `serde`, `serde_json`, `tokio` sync features to `crates/gateway/Cargo.toml`
  - [x] Implement `RouteEntry` struct for gateway in-memory config
  - [x] Fetch routes from `SKAILD2_CONTROL_PLANE_URL/api/internal/routes` at startup
  - [x] Start background task to refresh routes every 30 seconds
  - [x] Implement `match_route()` — best-fit by hostname + longest path_prefix
  - [x] For `public` route: reverse-proxy request to upstream_url using reqwest (headers + body pass-through)
  - [x] For `login_required` route: return `401 Unauthorized` with JSON `{"error": "Authentication required"}`
  - [x] For no matching route: return `404 Not Found` with JSON `{"error": "No route found"}`

- [x] Add integration tests (AC: 2, 3, 5)
  - [x] `test_create_route_requires_auth` — 401 when no session
  - [x] `test_create_route_persists` — 201, verifiable via `GET /api/applications/:id/routes`
  - [x] `test_list_routes_returns_created` — list returns new route
  - [x] `test_update_route_changes_access_mode` — PUT updates and GET reflects change
  - [x] `test_internal_routes_endoint_no_auth` — `GET /api/internal/routes` returns 200 without session

## Dev Notes

This story introduces the `routes` table and the gateway's first real routing logic. It establishes the pattern for access-mode enforcement that Epic 3 will extend with OIDC session validation.

### CRITICAL SCOPE BOUNDARY

**AC3 (IdP redirect) is NOT fully implemented in this story.** The gateway returns `401 Unauthorized` for `login_required` routes when no session is present. The actual OIDC redirect to the IdP login page is gated on Epic 3 (IdP configuration and OIDC flow). Do NOT attempt to implement OIDC flow here.

**AC4 (gateway picks up new routes):** Implemented via background polling every 30 seconds using a `tokio::time::interval`. This is sufficient for Story 2.2; a push-based config reload is deferred to Epic 5 (live config propagation, FR16).

### CRITICAL ARCHITECTURE REQUIREMENTS

1. **Routes are nested under Applications** [architecture.md § Data Architecture]
   - Route always has a foreign key `application_id → applications(id) ON DELETE CASCADE`
   - `host` is typically the application's hostname (pre-populated in the UI); path-based routing allows multiple routes per app
   - `access_mode` is the central policy toggle: `public` or `login_required` (role-restricted access comes in Epic 3)

2. **Internal gateway endpoint is unauthenticated** [architecture.md § Gateway–Control-plane communication]
   - `GET /api/internal/routes` returns a combined view of routes + their application's `upstream_url`
   - No session cookie required — this is an internal service-to-service call on the private Docker network
   - Do NOT expose this endpoint on a public port in production (the CORS config already limits cross-origin access; the Compose network limits container-to-container only)

3. **Auth middleware on all admin endpoints** [Story 2.1 patterns]
   - Reuse `require_auth(&session)` from `api/auth.rs` on all externally facing route endpoints
   - The pattern is identical to Story 2.1 applications endpoints

4. **enabled flag** [PRD § FR11]
   - Include `enabled BOOLEAN NOT NULL DEFAULT TRUE` from day one — enable/disable is a later story but the column must exist now

### CRITICAL LIBRARIES & VERSIONS

**Shared crate — no new crates needed** (sqlx 0.7, uuid 1, chrono 0.4, serde 1 already present).

**Control-plane — no new crates needed.**

**Gateway — add to `crates/gateway/Cargo.toml`:**
```toml
[dependencies]
shared = { path = "../shared" }
tokio = { version = "1.35", features = ["full"] }
axum = "0.7"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
dotenv = "0.15"
```

> **reqwest 0.12** requires tokio 1.x (already present). It uses `reqwest::Client` which is `Clone + Send + Sync` — wrap in `Arc` in `AppState`.

### FILE STRUCTURE — NEW FILES AND MODIFICATIONS

```
skaild2/
├── migrations/
│   └── 20260310000001_create_routes.sql          # NEW
│
├── crates/
│   ├── shared/
│   │   └── src/
│   │       └── models/
│   │           ├── mod.rs                        # MODIFY: pub mod route; pub use route::*;
│   │           └── route.rs                      # NEW
│   │
│   ├── control-plane/
│   │   └── src/
│   │       ├── main.rs                           # MODIFY: register 5 new routes
│   │       └── api/
│   │           ├── mod.rs                        # MODIFY: pub mod routes; pub use routes::*;
│   │           ├── setup.rs                      # MODIFY: add RouteError to AppError
│   │           └── routes.rs                     # NEW
│   │
│   └── gateway/
│       ├── Cargo.toml                            # MODIFY: add reqwest, serde, serde_json, tracing, dotenv
│       └── src/
│           └── main.rs                           # MODIFY: full routing implementation
│
├── admin-ui/
│   └── src/
│       ├── services/
│       │   └── api.ts                            # MODIFY: add Route types + CRUD functions
│       ├── pages/
│       │   └── ApplicationDetailPage.tsx         # MODIFY: add Routes section
│       └── components/
│           └── RouteForm.tsx                     # NEW
│
└── crates/control-plane/
    └── tests/
        └── routes_flow.rs                        # NEW
```

### DATABASE SCHEMA

**Migration: `migrations/20260310000001_create_routes.sql`**

```sql
CREATE TABLE IF NOT EXISTS routes (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id  UUID        NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    host            VARCHAR(255) NOT NULL,
    path_prefix     VARCHAR(255) NOT NULL DEFAULT '/',
    access_mode     VARCHAR(50)  NOT NULL DEFAULT 'login_required',
    enabled         BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT routes_access_mode_check CHECK (access_mode IN ('public', 'login_required'))
);

-- Allow multiple routes per application but enforce unique host+path per app
CREATE UNIQUE INDEX idx_routes_host_path ON routes(application_id, host, path_prefix);

-- Index for gateway route lookup by host
CREATE INDEX idx_routes_host ON routes(host);
```

> **Why VARCHAR + CHECK instead of PG ENUM?** SQLx runtime mode (non-compile-time) has ergonomics issues with custom PG ENUMs requiring type registration. A CHECK-constrained VARCHAR is simpler, just as safe at the DB level, and allows sqlx `query_as!` to work without special handling.

### RUST MODEL — `crates/shared/src/models/route.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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

/// Route stored in the database (access_mode as raw VARCHAR)
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

/// Route as returned by the API (access_mode as typed enum)
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

#[derive(Debug, Clone)]
pub struct CreateRouteInput {
    pub application_id: Uuid,
    pub host: String,
    pub path_prefix: Option<String>, // default "/"
    pub access_mode: AccessMode,
}

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
    #[error("Route with this host and path already exists")]
    DuplicateHostPath,
    #[error("Invalid access mode: {0}")]
    InvalidAccessMode(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}
```

**CRUD functions** (also in `route.rs`):

```rust
// create_route(): INSERT INTO routes ... RETURNING ...
// list_routes_by_application(pool, app_id): SELECT ... WHERE application_id = $1 AND enabled = true ORDER BY created_at ASC
// get_route_by_id(pool, id): SELECT ... WHERE id = $1
// update_route(pool, id, input): dynamic SET clause, RETURNING updated row
// list_all_enabled_routes_with_upstream(pool): JOIN routes + applications for gateway
```

For `list_all_enabled_routes_with_upstream`, use a JOIN:
```sql
SELECT r.id, r.host, r.path_prefix, r.access_mode, r.enabled, a.upstream_url
FROM routes r
JOIN applications a ON a.id = r.application_id
WHERE r.enabled = true AND a.enabled = true
ORDER BY LENGTH(r.path_prefix) DESC, r.host ASC
```

This gives the gateway exactly what it needs (sorted by path prefix length DESC so longest-match wins).

### CONTROL-PLANE API ENDPOINTS

```
POST /api/applications/:app_id/routes
Authorization: session cookie required
Request:  { "host": string, "path_prefix"?: string (default "/"), "access_mode": "public"|"login_required" }
Response: 201 { ...route }
Errors:   401 if not authenticated, 404 if app_id not found, 409 if duplicate host+path, 422 if host empty

GET /api/applications/:app_id/routes
Authorization: session cookie required
Response: 200 [ { ...route } ]
Errors:   401, 404 if app not found

GET /api/routes/:id
Authorization: session cookie required
Response: 200 { ...route }
Errors:   401, 404

PUT /api/routes/:id
Authorization: session cookie required
Request:  { "host"?: string, "path_prefix"?: string, "access_mode"?: string, "enabled"?: bool }
Response: 200 { ...route }
Errors:   401, 404, 409 if new host+path conflicts

GET /api/internal/routes
Authorization: NONE (internal service-to-service only)
Response: 200 [ { "id": uuid, "host": str, "path_prefix": str, "access_mode": str, "upstream_url": str } ]
```

### APPSTATE EXTENSION

No new fields needed in `AppState` for the control-plane (DB pool already there).

For the gateway, add a shared routes cache:

```rust
// crates/gateway/src/main.rs

use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GatewayRoute {
    pub id: String,
    pub host: String,
    pub path_prefix: String,
    pub access_mode: String, // "public" | "login_required"
    pub upstream_url: String,
}

pub type RoutesCache = Arc<RwLock<Vec<GatewayRoute>>>;
```

### GATEWAY IMPLEMENTATION SKELETON

```rust
// crates/gateway/src/main.rs

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

// GatewayRoute, RoutesCache as above

#[derive(Clone)]
struct AppState {
    routes: RoutesCache,
    client: Client,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let control_plane_url = std::env::var("SKAILD2_CONTROL_PLANE_URL")
        .unwrap_or_else(|_| "http://control-plane:8080".to_string());

    let client = Client::new();
    let routes: RoutesCache = Arc::new(RwLock::new(vec![]));

    // Initial load
    load_routes(&client, &control_plane_url, &routes).await;

    // Background refresh every 30 seconds
    let bg_client = client.clone();
    let bg_url = control_plane_url.clone();
    let bg_routes = routes.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            load_routes(&bg_client, &bg_url, &bg_routes).await;
        }
    });

    let state = AppState { routes, client };

    let app = Router::new()
        .route("/health", axum::routing::get(health_check))
        .fallback(any(proxy_handler))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 80));
    tracing::info!("Gateway listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn load_routes(client: &Client, base_url: &str, cache: &RoutesCache) {
    let url = format!("{}/api/internal/routes", base_url);
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(routes) = resp.json::<Vec<GatewayRoute>>().await {
                tracing::info!("Loaded {} routes from control-plane", routes.len());
                *cache.write().await = routes;
            }
        }
        Ok(resp) => tracing::warn!("Control-plane returned {} for routes", resp.status()),
        Err(e) => tracing::error!("Failed to load routes from control-plane: {}", e),
    }
}

fn match_route<'a>(routes: &'a [GatewayRoute], host: &str, path: &str) -> Option<&'a GatewayRoute> {
    // Routes are pre-sorted by path_prefix length DESC from control-plane
    routes.iter().find(|r| {
        r.host == host && path.starts_with(&r.path_prefix)
    })
}

async fn proxy_handler(
    State(state): State<AppState>,
    req: Request,
) -> Response {
    let host = req
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .map(|h| h.split(':').next().unwrap_or(h))
        .unwrap_or("");
    let path = req.uri().path();

    let routes = state.routes.read().await;
    let Some(route) = match_route(&routes, host, path) else {
        return (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({"error": "No route found"}))).into_response();
    };

    match route.access_mode.as_str() {
        "login_required" => {
            // TODO (Epic 3): check OIDC session cookie; redirect to IdP if absent
            (StatusCode::UNAUTHORIZED, axum::Json(serde_json::json!({"error": "Authentication required"}))).into_response()
        }
        "public" => {
            let target_url = format!("{}{}", route.upstream_url.trim_end_matches('/'), req.uri());
            drop(routes); // release read lock before async I/O
            forward_request(state.client, req, &target_url).await
        }
        _ => (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({"error": "Invalid access mode"}))).into_response(),
    }
}

async fn forward_request(client: Client, req: Request, target_url: &str) -> Response {
    // Build reqwest request from axum request
    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .unwrap_or(reqwest::Method::GET);
    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await.unwrap_or_default();

    let mut rb = client.request(method, target_url).body(body);
    for (name, value) in &headers {
        // Skip hop-by-hop headers
        let n = name.as_str();
        if matches!(n, "host" | "connection" | "transfer-encoding" | "te" | "trailer" | "upgrade") {
            continue;
        }
        rb = rb.header(name, value);
    }

    match rb.send().await {
        Ok(upstream_resp) => {
            let status = axum::http::StatusCode::from_u16(upstream_resp.status().as_u16())
                .unwrap_or(StatusCode::BAD_GATEWAY);
            let resp_headers = upstream_resp.headers().clone();
            let resp_body = upstream_resp.bytes().await.unwrap_or_default();

            let mut response = Response::new(Body::from(resp_body));
            *response.status_mut() = status;
            for (name, value) in &resp_headers {
                let n = name.as_str();
                if matches!(n, "connection" | "transfer-encoding") { continue; }
                response.headers_mut().insert(name, value.clone());
            }
            response
        }
        Err(e) => {
            tracing::error!("Upstream proxy error: {}", e);
            (StatusCode::BAD_GATEWAY, axum::Json(serde_json::json!({"error": "Bad gateway"}))).into_response()
        }
    }
}
```

### CONTROL-PLANE `routes.rs` HANDLER PATTERN

Follow `applications.rs` exactly. Example for `create_route`:

```rust
pub async fn create_route(
    session: Session,
    State(state): State<AppState>,
    Path(app_id): Path<Uuid>,
    Json(payload): Json<CreateRouteRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_auth(&session).await?;

    // Verify application exists first
    shared::get_application_by_id(&state.db_pool, app_id).await?;

    let route = shared::create_route(
        &state.db_pool,
        CreateRouteInput {
            application_id: app_id,
            host: payload.host,
            path_prefix: payload.path_prefix,
            access_mode: payload.access_mode
                .as_deref()
                .map(AccessMode::try_from)
                .transpose()
                .map_err(AppError::RouteError)? // parse access_mode string
                .unwrap_or(AccessMode::LoginRequired),
        },
    ).await?;

    Ok((StatusCode::CREATED, Json(RouteResponse::from(route))))
}
```

The `AppError` conversion from `ApplicationError::NotFound` already returns 404, so calling `get_application_by_id` naturally returns the right error.

### ROUTE REGISTRATION IN `main.rs`

Add after existing application routes:

```rust
// Route endpoints
.route("/api/applications/:app_id/routes",
    get(api::list_routes).post(api::create_route))
.route("/api/routes/:id",
    get(api::get_route).put(api::update_route))
// Internal gateway endpoint (no auth)
.route("/api/internal/routes", get(api::get_internal_routes))
```

### ADMIN UI — ROUTE TYPES AND API FUNCTIONS

Add to `admin-ui/src/services/api.ts`:

```typescript
// Route types
export interface Route {
  id: string;
  application_id: string;
  host: string;
  path_prefix: string;
  access_mode: 'public' | 'login_required';
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateRouteInput {
  host: string;
  path_prefix?: string;
  access_mode: 'public' | 'login_required';
}

export interface UpdateRouteInput {
  host?: string;
  path_prefix?: string;
  access_mode?: 'public' | 'login_required';
  enabled?: boolean;
}

// Route API functions
export const getRoutes = async (appId: string): Promise<Route[]> => {
  const response = await api.get<Route[]>(`/api/applications/${appId}/routes`);
  return response.data;
};

export const createRoute = async (appId: string, data: CreateRouteInput): Promise<Route> => {
  const response = await api.post<Route>(`/api/applications/${appId}/routes`, data);
  return response.data;
};

export const updateRoute = async (id: string, data: UpdateRouteInput): Promise<Route> => {
  const response = await api.put<Route>(`/api/routes/${id}`, data);
  return response.data;
};
```

### ADMIN UI — `RouteForm.tsx` STRUCTURE

```
- Host: text input, required, pre-filled with application.hostname
- Path Prefix: text input, optional, default "/" placeholder
- Access Mode: radio or select
    - "public" → label "Public (no login required)"
    - "login_required" → label "Login required (default)" [default selected]
- Submit: "Save Route" (mc-button-primary)
- Cancel: text link
- Inline field-level error display
```

### ADMIN UI — `ApplicationDetailPage.tsx` EXTENSION

Below the existing application details section, add a "Routes" section:

```tsx
{/* Routes section */}
<section className="space-y-4">
  <div className="flex items-center justify-between">
    <h2 className="text-lg font-semibold text-slate-100">Routes</h2>
    <button
      onClick={() => setAddingRoute(true)}
      className="mc-button-primary px-4 py-2 text-sm"
    >
      Add Route
    </button>
  </div>

  {/* Route list or empty state */}
  {routes.length === 0 ? (
    <p className="text-slate-400 text-sm">No routes yet. Add your first one.</p>
  ) : (
    <table className="w-full text-sm">
      <thead>
        <tr className="text-slate-400 text-xs uppercase tracking-wide text-left">
          <th className="pb-2">Host</th>
          <th className="pb-2">Path Prefix</th>
          <th className="pb-2">Access Mode</th>
          <th className="pb-2">Status</th>
        </tr>
      </thead>
      <tbody className="space-y-1">
        {routes.map(route => (
          <RouteRow key={route.id} route={route} onEdit={...} />
        ))}
      </tbody>
    </table>
  )}

  {addingRoute && (
    <RouteForm
      applicationHostname={application.hostname}
      onSubmit={handleAddRoute}
      onCancel={() => setAddingRoute(false)}
    />
  )}
</section>
```

Access mode badge styling:
- `public`: `bg-blue-500/20 text-blue-400`
- `login_required`: `bg-yellow-500/20 text-yellow-400`

### TESTING REQUIREMENTS

**Unit Tests (shared crate — in `route.rs`):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_mode_parse_valid() {
        assert_eq!(AccessMode::try_from("public").unwrap(), AccessMode::Public);
        assert_eq!(AccessMode::try_from("login_required").unwrap(), AccessMode::LoginRequired);
    }

    #[test]
    fn test_access_mode_parse_invalid() {
        assert!(AccessMode::try_from("admin_only").is_err());
        assert!(AccessMode::try_from("").is_err());
    }

    #[test]
    fn test_access_mode_as_str() {
        assert_eq!(AccessMode::Public.as_str(), "public");
        assert_eq!(AccessMode::LoginRequired.as_str(), "login_required");
    }
}
```

**Integration Tests (control-plane crate — new file `tests/routes_flow.rs`):**
```rust
// Use #[file_serial] on every test (same as applications_flow.rs and auth_flow.rs)
use serial_test::file_serial;

#[tokio::test]
#[file_serial]
async fn test_create_route_requires_auth() { ... }   // 401 without session

#[tokio::test]
#[file_serial]
async fn test_create_route_persists() { ... }         // 201 + GET returns it

#[tokio::test]
#[file_serial]
async fn test_list_routes_returns_created() { ... }   // list after create

#[tokio::test]
#[file_serial]
async fn test_update_route_changes_access_mode() { ... }  // PUT updates + GET reflects

#[tokio::test]
#[file_serial]
async fn test_internal_routes_endpoint_no_auth() { ... }  // 200 without session
```

### PREVIOUS STORY LEARNINGS (from 2.1 Dev Agent Record)

✅ **Patterns to replicate:**
- `#[file_serial]` on ALL integration tests (prevents cross-binary DB races between auth_flow, applications_flow, and now routes_flow)
- `axum::body::to_bytes(body, usize::MAX)` — not `hyper::body::to_bytes` (hyper v1 API change)
- `AppError` extensions follow existing `From<XxxError>` + `IntoResponse` match arms pattern in `setup.rs`
- Export ALL new handler symbols from `api/mod.rs` with `pub use routes::*;`
- Register routes in `main.rs` — do NOT forget the `axum::routing::put` import if adding `put()`

⚠️ **Issues to avoid (from 2.1 debug log):**
- Do NOT use PG ENUM type for `access_mode` — SQLx runtime mode doesn't support custom PG ENUMs without manual type registration. Use `VARCHAR(50)` + `CHECK` constraint.
- The `api/auth.rs` `ADMIN_SESSION_KEY` and `require_auth()` are already `pub(crate)` — do NOT re-declare them
- The internal routes endpoint (`/api/internal/routes`) must NOT call `require_auth()` — it is a service-to-service endpoint used by the gateway
- When calling `get_application_by_id()` in route handlers, `ApplicationError::NotFound` already maps to `404` via `AppError::ApplicationError(ApplicationError::NotFound)` — leverage this for free 404 when app doesn't exist

### IMPLEMENTATION SEQUENCE

1. **Migration** — create SQL, verify column names match struct fields exactly
2. **Shared model** — `Route` / `RouteRow`, `AccessMode`, `RouteError`, CRUD functions + unit tests
3. **Control-plane API** — 5 endpoints, extend AppError (RouteError variants + IntoResponse arms), register routes
4. **Admin UI** — api.ts additions → RouteForm → extend ApplicationDetailPage
5. **Gateway** — update Cargo.toml → implement load_routes(), match_route(), proxy_handler(), forward_request()
6. **Integration tests** — `routes_flow.rs` with 5 test cases, all `#[file_serial]`
7. **Manual smoke test** — create route, verify it appears in app detail view, verify gateway returns 401 for login_required route and proxies for public route

### REFERENCES

- Story requirements: [_bmad-output/planning-artifacts/epics.md — Epic 2, Story 2.2](../../planning-artifacts/epics.md)
- PRD FRs covered: FR8 (route definitions), FR9 (route access modes), FR11 (enable/disable placeholder), FR12 (protocol routing foundation) [_bmad-output/planning-artifacts/prd.md]
- Architecture — Gateway/control-plane separation, data model, security patterns: [_bmad-output/planning-artifacts/architecture.md § Data Architecture, API & Communication Patterns, Authentication & Security]
- Established patterns from: [_bmad-output/implementation-artifacts/2-1-register-first-upstream-application.md]
- UX system: [_bmad-output/planning-artifacts/ux-design-specification.md § Visual Language: Mermaidcore]
- Docker Compose gateway env vars: [docker-compose.dev.yml — gateway.environment.SKAILD2_CONTROL_PLANE_URL]

## Code Review Record

### Reviewer

Amelia (Dev Agent) — GitHub Copilot (Claude Sonnet 4.6) — 2026-03-10

### Findings Fixed

| ID | Severity | File | Fix Applied |
|----|----------|------|--------------|
| H2 | HIGH | `crates/gateway/src/main.rs` | Changed `to_bytes(body, usize::MAX)` → `to_bytes(body, 64 * 1024 * 1024)` to prevent memory exhaustion DoS |
| M1 | MEDIUM | `crates/shared/src/models/route.rs` | Trim `input.host` before binding to SQL — host with surrounding spaces was stored verbatim and never matched gateway routing |
| M2 | MEDIUM | `admin-ui/src/pages/ApplicationDetailPage.tsx` | Empty state text corrected to `"No routes yet. Add your first one."` per story spec |
| M3 | MEDIUM | `admin-ui/src/components/RouteForm.tsx` | Added descriptive comment to empty `catch` block clarifying intent |
| M4 | MEDIUM | `crates/shared/src/models/route.rs` | Removed unnecessary `Deserialize` derive from `RouteWithUpstream` — struct is only serialized to JSON |

### Findings Kept As-Is

| ID | Severity | Rationale |
|----|----------|-----------|
| H1 | HIGH | `RouteError::ApplicationNotFound` is returned by `create_route()` on FK violation — not dead code; pre-validation in handler is belt-and-suspenders |
| L1 | LOW | Same as H1 — variant is reachable via DB FK path |
| L2 | LOW | `X-Forwarded-For` injection deferred — upstream identity is not required at this story scope |

### Test Results Post-Fix

- `cargo test -p shared --lib`: **30/30 passed**
- `cargo test -p control-plane --test routes_flow`: **5/5 passed**
- `cargo check -p gateway`: **clean**

## Dev Agent Record

### Agent Model Used

Amelia (Dev Agent) — GitHub Copilot (Claude Sonnet 4.6)

### Debug Log References

- Used `VARCHAR(50) + CHECK` for `access_mode` column (not PG ENUM) — avoids sqlx runtime mode registration issues (established in Story 2.1)
- `MethodRouter::put()` is a method on an existing MethodRouter; no need to import standalone `put` from `axum::routing`
- `axum::body::to_bytes(body, usize::MAX)` required in gateway proxy handler (axum 0.7 / http v1 API)
- `reqwest 0.12` uses same `http` crate types as axum 0.7 — header name/value clones work directly across both APIs

### Completion Notes List

- All 5 AC satisfied; AC3/4 at story scope (gateway returns 401 for `login_required`; OIDC redirect deferred to Epic 3)
- `#[allow(dead_code)]` added to `id` field in gateway `GatewayRoute` — included for future use but not consumed in routing logic
- `test_internal_routes_endpoint_no_auth` confirms the unauthenticated internal endpoint works correctly
- `applications_flow` and `auth_flow` tests continue to pass (8+6 = 14 existing tests unaffected)

### File List

**New files:**
- `migrations/20260310000001_create_routes.sql`
- `crates/shared/src/models/route.rs`
- `crates/control-plane/src/api/routes.rs`
- `admin-ui/src/components/RouteForm.tsx`
- `crates/control-plane/tests/routes_flow.rs`

**Modified files:**
- `crates/shared/src/models/mod.rs` — added route module
- `crates/control-plane/src/api/mod.rs` — exported routes module
- `crates/control-plane/src/api/setup.rs` — added RouteError to AppError + From impl + IntoResponse arms
- `crates/control-plane/src/main.rs` — registered 3 new route paths (5 endpoints total)
- `admin-ui/src/services/api.ts` — added Route, CreateRouteInput, UpdateRouteInput types + getRoutes/createRoute/updateRoute functions
- `admin-ui/src/pages/ApplicationDetailPage.tsx` — added Routes section with RouteForm, route table, empty state
- `crates/gateway/Cargo.toml` — added reqwest 0.12, serde, serde_json, tracing, tracing-subscriber, dotenv
- `crates/gateway/src/main.rs` — full proxy implementation: route cache, background refresh, match_route, proxy_handler, forward_request
