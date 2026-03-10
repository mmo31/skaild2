# Story 2.1: Register First Upstream Application

Status: ready-for-dev

## Story

As a homelab admin,
I want to register an upstream HTTP service and bind it to a hostname under my wildcard domain,
so that skaild2 knows where to send proxied traffic for that application.

## Acceptance Criteria

1. **Given** I am signed in as an admin, **when** I navigate to the Applications section in the admin UI and choose to add a new application, **then** I can enter at least an application name, an upstream URL (including scheme and port), and a desired external hostname under the configured wildcard domain.

2. **Given** I complete the form with a valid upstream URL and hostname, **when** I save the new application, **then** the application is persisted in the control-plane database and appears in the Applications list with its configured upstream and hostname.

3. **Given** I have registered at least one application, **when** I return to the Applications list later or after restarting the stack, **then** the application still appears with the same upstream and hostname configuration.

4. **Given** I have an existing application registered, **when** I edit its upstream URL or hostname using the admin UI, **then** the updated values are saved and reflected consistently wherever the application is shown.

## Tasks / Subtasks

- [ ] Add database migration for applications table (AC: 2, 3)
  - [ ] Create `20260309000001_create_applications.sql` in `migrations/`
  - [ ] Define `applications` table: id, name, upstream_url, hostname, enabled, created_at, updated_at
  - [ ] Add unique constraint on hostname

- [ ] Implement Application model and persistence in shared crate (AC: 2, 3, 4)
  - [ ] Create `crates/shared/src/models/application.rs` with `Application` struct (sqlx `FromRow`, Serialize, Deserialize)
  - [ ] Add `CreateApplicationInput` and `UpdateApplicationInput` structs
  - [ ] Implement `create_application()` async fn with URL and hostname validation
  - [ ] Implement `list_applications()` async fn
  - [ ] Implement `get_application_by_id()` async fn
  - [ ] Implement `update_application()` async fn
  - [ ] Add `ApplicationError` enum (thiserror)
  - [ ] Register `application` module in `shared/src/models/mod.rs`
  - [ ] Add unit tests for validation logic

- [ ] Build control-plane API endpoints (AC: 1, 2, 3, 4)
  - [ ] Create `crates/control-plane/src/api/applications.rs`
  - [ ] `POST /api/applications` — create a new application (auth required)
  - [ ] `GET /api/applications` — list all applications (auth required)
  - [ ] `GET /api/applications/:id` — get a single application (auth required)
  - [ ] `PUT /api/applications/:id` — update name, upstream_url, or hostname (auth required)
  - [ ] Add `ApplicationError` to control-plane `AppError` mapping
  - [ ] Export new handlers from `api/mod.rs`
  - [ ] Register routes in `main.rs`

- [ ] Build Applications UI (AC: 1, 2, 3, 4)
  - [ ] Add `getApplications()`, `createApplication()`, `updateApplication()` to `admin-ui/src/services/api.ts`
  - [ ] Create `admin-ui/src/pages/ApplicationsPage.tsx` — list with "Add Application" CTA
  - [ ] Create `admin-ui/src/pages/ApplicationDetailPage.tsx` — view + edit inline or modal
  - [ ] Create `admin-ui/src/components/ApplicationForm.tsx` — shared form for create/edit
  - [ ] Add `/applications` and `/applications/:id` routes in `App.tsx`
  - [ ] Add **Applications** entry to the left-rail navigation in `DashboardPage.tsx` (between Dashboard and Routes/Identity)

- [ ] Add integration tests (AC: 2, 3, 4)
  - [ ] `create_application` succeeds and persists
  - [ ] `list_applications` returns created application after restart simulation
  - [ ] `update_application` updates fields correctly
  - [ ] Unauthenticated requests to `/api/applications` return 401

- [ ] Update documentation (AC: 1)
  - [ ] Document Applications CRUD in README or DOCKER.md
  - [ ] Note wildcard hostname requirement

## Dev Notes

This story introduces the first domain entity beyond `admins` and establishes the CRUD pattern that all subsequent Epic 2 and Epic 3 stories will follow. **Do not skip tests or leave stub implementations.**

### CRITICAL ARCHITECTURE REQUIREMENTS

**From Architecture Document:**

1. **Applications are the core abstraction** [architecture.md § Data Architecture]
   - `applications` table is the anchor for routes, policies, certs, and audit events in later epics
   - `hostname` must be unique across all applications — enforced both at DB level (unique constraint) and in Rust validation
   - `upstream_url` must include scheme and host, e.g. `http://homelab-server:8080`; validate with Rust's `url` crate

2. **Auth middleware on all new endpoints** [architecture.md § Auth & Security, story 1.1 patterns]
   - All `/api/applications/**` endpoints require a valid session (checked via `tower-sessions`)
   - Reuse the auth middleware/extractor pattern established in `api/auth.rs` from Story 1.1
   - Return `401 Unauthorized` with `{"error": "Unauthorized"}` for unauthenticated requests

3. **Control-plane is the single source of truth** [architecture.md § Single Source of Truth]
   - Gateway will later read application config from control-plane; the DB schema must be clean and normalized
   - Do NOT add gateway-specific columns to the applications table in this story (keep scope tight)

4. **Enabled flag** [PRD § FR11]
   - Include `enabled BOOLEAN NOT NULL DEFAULT TRUE` in the migration — FR11 (enable/disable) is a later story, but the column must exist now to avoid a breaking migration later

### CRITICAL LIBRARIES & VERSIONS

**No new Rust crates needed** — all required crates are already in `control-plane/Cargo.toml`:
- `sqlx 0.7` — DB queries and FromRow
- `uuid 1` — primary key
- `chrono 0.4` — timestamps
- `serde 1` — JSON serialization
- `axum 0.7` — handler pattern (use `Json`, `Path`, `State`, `Extension` extractors)

**One new crate needed** in `shared/Cargo.toml` for URL validation:
```toml
url = { version = "2", features = ["serde"] }
```

**Admin UI — no new npm packages needed**: `axios` and `react-router-dom` already installed.

### FILE STRUCTURE — NEW FILES AND MODIFICATIONS

```
skaild2/
├── migrations/
│   └── 20260309000001_create_applications.sql   # NEW
│
├── crates/
│   ├── shared/
│   │   ├── Cargo.toml                            # MODIFY: add url crate
│   │   └── src/
│   │       └── models/
│   │           ├── mod.rs                        # MODIFY: pub mod application;
│   │           └── application.rs                # NEW
│   │
│   └── control-plane/
│       └── src/
│           ├── main.rs                           # MODIFY: register 4 new routes
│           └── api/
│               ├── mod.rs                        # MODIFY: pub mod applications; pub use applications::*;
│               └── applications.rs               # NEW
│
└── admin-ui/
    └── src/
        ├── App.tsx                               # MODIFY: add /applications routes
        ├── services/
        │   └── api.ts                            # MODIFY: add 3 application functions
        ├── pages/
        │   ├── ApplicationsPage.tsx              # NEW
        │   └── ApplicationDetailPage.tsx         # NEW
        └── components/
            └── ApplicationForm.tsx               # NEW
```

### DATABASE SCHEMA

**Migration: `migrations/20260309000001_create_applications.sql`**

```sql
CREATE TABLE IF NOT EXISTS applications (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    upstream_url TEXT NOT NULL,
    hostname    VARCHAR(255) NOT NULL,
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_applications_hostname ON applications(hostname);
```

**Do NOT** add foreign keys to routes, policies, or certs yet — those tables don't exist.

### API ENDPOINT SPECIFICATIONS

```
POST /api/applications
Authorization: session cookie required
Request:  { "name": string, "upstream_url": string, "hostname": string }
Response: 201 { "id": uuid, "name": string, "upstream_url": string, "hostname": string, "enabled": bool, "created_at": timestamp, "updated_at": timestamp }
Errors:   400 if upstream_url not a valid URL, 409 if hostname already taken, 422 if name empty

GET /api/applications
Authorization: session cookie required
Response: 200 [ { ...application } ]

GET /api/applications/:id
Authorization: session cookie required
Response: 200 { ...application }
Errors:   404 if not found

PUT /api/applications/:id
Authorization: session cookie required
Request:  { "name"?: string, "upstream_url"?: string, "hostname"?: string }  (all fields optional, at least one required)
Response: 200 { ...updated application }
Errors:   400 if upstream_url invalid, 404 if not found, 409 if new hostname already taken
```

### RUST MODEL PATTERN — Follow admin.rs exactly

```rust
// crates/shared/src/models/application.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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

#[derive(Debug, Clone)]
pub struct CreateApplicationInput {
    pub name: String,
    pub upstream_url: String,
    pub hostname: String,
}

#[derive(Debug, Clone)]
pub struct UpdateApplicationInput {
    pub name: Option<String>,
    pub upstream_url: Option<String>,
    pub hostname: Option<String>,
}

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
```

**Validate `upstream_url`** using the `url` crate:
```rust
use url::Url;

fn validate_upstream_url(raw: &str) -> Result<(), ApplicationError> {
    let parsed = Url::parse(raw)
        .map_err(|e| ApplicationError::InvalidUpstreamUrl(e.to_string()))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(ApplicationError::InvalidUpstreamUrl(
            "scheme must be http or https".to_string(),
        ));
    }
    Ok(())
}
```

### AUTH EXTRACTOR PATTERN — Reuse from Story 1.1

In `api/auth.rs` from Story 1.1, session-based auth was implemented via a tower-sessions extractor. The same session extractor (checking for `admin_id` key in the session) must wrap all application endpoints. Example handler signature:

```rust
// In api/applications.rs
pub async fn list_applications(
    session: Session,
    State(state): State<AppState>,
) -> Result<Json<Vec<Application>>, AppError> {
    // 1. Check session has admin_id (reuse require_auth pattern from auth.rs)
    let _admin_id = require_auth(&session).await?;
    // 2. Fetch from DB
    ...
}
```

Check how `require_auth` (or equivalent) is implemented in `api/auth.rs` — use the exact same pattern. Do NOT duplicate auth logic.

### AXUM HANDLER PATTERN — established in auth.rs / setup.rs

```rust
// Route registration in main.rs — add after existing auth routes:
.route("/api/applications",     get(api::list_applications).post(api::create_application))
.route("/api/applications/:id", get(api::get_application).put(api::update_application))
```

### ADMIN UI PATTERNS — Follow Story 1.1 Mermaidcore conventions

**Navigation update in `DashboardPage.tsx`:**
Add an "Applications" item to the nav array, positioned between Dashboard and the existing items. Use an appropriate Heroicons icon (e.g., `ServerIcon` or `CubeIcon`).

**ApplicationsPage.tsx — structure:**
```
- Page title: "Applications" (text-dark, large)
- CTA button: "Add Application" (mc-button-primary, top-right)
- Table or card list on mc-surface / glassmorphic background:
  - Columns: Name | Hostname | Upstream URL | Status (enabled/disabled) | Actions (Edit)
  - Empty state: "No applications yet. Add your first one." with CTA
```

**ApplicationForm.tsx — fields:**
```
- Name: text input, required, placeholder "My Home Assistant"
- Upstream URL: text input, required, placeholder "http://192.168.1.10:8123"
- Hostname: text input, required, placeholder "homeassistant.yourdomain.com"
  - Note: pre-fill with configured wildcard domain later; for now, free text
- Submit button: "Save Application" (mc-button-primary)
- Cancel button: text link
- Display server-side errors inline under the relevant field
```

**api.ts additions:**
```typescript
// Application types
export interface Application {
  id: string;
  name: string;
  upstream_url: string;
  hostname: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateApplicationInput {
  name: string;
  upstream_url: string;
  hostname: string;
}

export interface UpdateApplicationInput {
  name?: string;
  upstream_url?: string;
  hostname?: string;
}

// Functions to add to existing api.ts
export const getApplications = () =>
  apiClient.get<Application[]>('/api/applications').then(r => r.data);

export const createApplication = (data: CreateApplicationInput) =>
  apiClient.post<Application>('/api/applications', data).then(r => r.data);

export const updateApplication = (id: string, data: UpdateApplicationInput) =>
  apiClient.put<Application>(`/api/applications/${id}`, data).then(r => r.data);
```

### TESTING REQUIREMENTS

**Unit Tests (shared crate — in `application.rs`):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_upstream_url() {
        assert!(validate_upstream_url("http://192.168.1.10:8080").is_ok());
        assert!(validate_upstream_url("https://192.168.1.10:443").is_ok());
    }

    #[test]
    fn test_invalid_upstream_url() {
        assert!(validate_upstream_url("ftp://server").is_err());
        assert!(validate_upstream_url("not-a-url").is_err());
        assert!(validate_upstream_url("").is_err());
    }
}
```

**Integration Tests (control-plane crate — new file `tests/applications_flow.rs`):**
```rust
#[tokio::test]
async fn test_create_application_requires_auth()      { ... } // 401 when no session
#[tokio::test]
async fn test_create_application_persists()           { ... } // 201, verifiable via GET
#[tokio::test]
async fn test_list_applications_returns_created()     { ... } // GET returns the new item
#[tokio::test]
async fn test_update_application_changes_values()    { ... } // PUT updates and GET reflects change
#[tokio::test]
async fn test_duplicate_hostname_returns_409()        { ... } // Unique constraint surfaced correctly
```

### PREVIOUS STORY LEARNINGS (from 1.1 Dev Agent Record)

✅ **Patterns to replicate:**
- Edition = `"2021"` in all Cargo.toml files
- `AppError` enum in control-plane with `IntoResponse` impl — extend it with `ApplicationError` variant, map to 400/404/409 as appropriate
- `thiserror` for domain errors in shared crate
- `FromRow` + `Serialize/Deserialize` on model structs
- CORS already configured for PUT and DELETE — no changes needed

⚠️ **Issues to avoid (from 1.1 review):**
- **Do NOT leave stub implementations** — all CRUD functions must hit the real DB
- **Do NOT forget to register the new `application` module** in `shared/src/models/mod.rs`
- **Do NOT forget to export** `list_applications`, `create_application`, `get_application`, `update_application` from `api/mod.rs`
- The `api/auth.rs` extractor for session auth must be reused — look at its exact signature before copy-pasting
- No new environment variables required for this story

### IMPLEMENTATION SEQUENCE

1. **Migration first** — create SQL, verify with `sqlx migrate run` or let control-plane run it on startup
2. **Shared model** — `Application` struct, validation, CRUD functions + unit tests
3. **Control-plane API** — 4 endpoints, extend `AppError`, register routes in `main.rs`
4. **Admin UI** — api.ts additions → ApplicationForm → ApplicationsPage → ApplicationDetailPage → App.tsx routes → nav update
5. **Integration tests** — cover all 5 scenarios
6. **Manual smoke test** — create an application, restart stack, verify it persists

### REFERENCES

- Story requirements: [_bmad-output/planning-artifacts/epics.md — Epic 2, Story 2.1](../../planning-artifacts/epics.md)
- PRD FRs covered: FR7, FR11 (enabled flag placeholder), FR24 (hostname) [_bmad-output/planning-artifacts/prd.md]
- Architecture — Data model, API patterns, Security: [_bmad-output/planning-artifacts/architecture.md § Data Architecture, API & Communication Patterns]
- UX design system: [_bmad-output/planning-artifacts/ux-design-specification.md § Visual Language: Mermaidcore]
- Established patterns from: [_bmad-output/implementation-artifacts/1-1-single-host-compose-deployment-with-first-admin-login.md]

## Dev Agent Record

### Agent Model Used

_to be filled by dev agent_

### Debug Log References

### Completion Notes List

### File List

