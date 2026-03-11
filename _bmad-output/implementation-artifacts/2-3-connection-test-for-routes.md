# Story 2.3: Connection Test for Routes

Status: review

## Story

As a homelab admin,
I want to run a connection test for a route,
so that I can quickly see whether upstream connectivity, TLS, and basic auth wiring are correct before exposing the route broadly.

## Acceptance Criteria

1. **Given** at least one route exists for a registered application, **when** I trigger a connection test from the route's detail view or actions menu, **then** the system attempts to contact the configured upstream using the stored URL and reports success or failure clearly in the UI.

2. **Given** the upstream is reachable and returns a successful response, **when** the connection test completes, **then** I see a success status that indicates the route is reachable and ready for traffic.

3. **Given** the upstream URL is invalid, the host cannot be resolved, or TLS handshakes fail, **when** the connection test runs, **then** I see a failed status along with a short, actionable error description (for example, "DNS resolution failed", "TLS certificate validation failed", or "connection timed out").

4. **Given** the route requires login through an identity provider, **when** I run the connection test, **then** the test verifies that the auth flow for that route is configured (required IdP is set) and surfaces a clear error if mandatory auth configuration is missing.

## Tasks / Subtasks

- [x] Add `reqwest` to control-plane and extend `AppState` with `http_client` (AC: 1, 2, 3)
  - [x] In `crates/control-plane/Cargo.toml`: add `reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }` under `[dependencies]`
  - [x] In `crates/control-plane/Cargo.toml`: add `wiremock = "0.6"` under `[dev-dependencies]`
  - [x] In `crates/control-plane/src/state.rs`: add `pub http_client: reqwest::Client` field; extend `AppState::new()` to build a client with a 5s `connection_verbose(false)` default; keep `Clone` derive (reqwest::Client is cheaply cloneable internally)

- [x] Add `get_route_with_upstream_by_id` helper to shared crate (AC: 1, 2, 3, 4)
  - [x] In `crates/shared/src/models/route.rs`: add `async fn get_route_with_upstream_by_id(pool, id: Uuid) -> Result<RouteWithUpstream, RouteError>` using a single JOIN query mirroring `list_all_enabled_routes_with_upstream` but filtered by `routes.id = $1` (no `enabled` filter — admin should be able to test disabled routes too)
  - [x] Re-export the new function in `crates/shared/src/lib.rs`

- [x] Implement `test_route` handler in control-plane (AC: 1, 2, 3, 4)
  - [x] In `crates/control-plane/src/api/routes.rs`: add `ConnectionTestResult` response struct (fields: `status: String`, `http_status: Option<u16>`, `latency_ms: Option<u64>`, `error_kind: Option<String>`, `error_message: Option<String>`, `auth_check: Option<AuthCheckResult>`)
  - [x] Add `AuthCheckResult` struct (fields: `configured: bool`, `message: String`)
  - [x] Implement `test_route` handler: `POST /api/routes/:id/test` — requires auth, looks up `RouteWithUpstream` by id, constructs test URL as `{upstream_url}{path_prefix}`, fires `http_client.get(url).timeout(Duration::from_secs(5)).send().await`, classifies the outcome, returns `ConnectionTestResult`
  - [x] Error classification logic:
    - If `reqwest::Error::is_timeout()` → `error_kind: "timeout"`, message: `"Connection timed out after 5 seconds"`
    - If `reqwest::Error::is_connect()` and error string contains "dns" (case-insensitive) → `error_kind: "dns"`, message: `"DNS resolution failed for host '{host}'"` (extract host from URL)
    - If `reqwest::Error::is_connect()` and error string contains "tls" or "certificate" (case-insensitive) → `error_kind: "tls"`, message: `"TLS certificate validation failed"`
    - If `reqwest::Error::is_connect()` (generic) → `error_kind: "connection"`, message: `"Connection refused or host unreachable"`
    - On HTTP response (any status code) → `status: "ok"`, `http_status: response.status().as_u16()`, `latency_ms: elapsed`
    - Auth check (always run if `access_mode == "login_required"`): for this story, always return `AuthCheckResult { configured: false, message: "No identity provider configured — set one up in Epic 3 to enable auth validation" }`. Once IdP is wired, this block will query the IDP table.
  - [x] Re-export `test_route` from `crates/control-plane/src/api/mod.rs`

- [x] Register the new endpoint in the router (AC: 1)
  - [x] In `crates/control-plane/src/main.rs`: initialize `http_client` via `reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build().unwrap()` and pass it to `AppState::new(pool, http_client)`; add route `.route("/api/routes/:id/test", post(api::test_route))`
  - [x] Fix `AppState::new()` signature to accept both `db_pool` and `http_client`

- [x] Write integration tests (AC: 1, 2, 3, 4)
  - [x] Create `crates/control-plane/tests/connection_test_flow.rs`
  - [x] Update `create_test_app()` helper in that file to build an `AppState` with a real `reqwest::Client` (same pattern as `routes_flow.rs` but also initialising `http_client`)
  - [x] `test_connection_test_requires_auth` — trigger `POST /api/routes/{id}/test` without session cookie → `401`
  - [x] `test_connection_test_unreachable` — create route with upstream `http://127.0.0.1:19991` (port that is never bound), trigger test → `200` with `{ status: "error", error_kind: "connection" }`
  - [x] `test_connection_test_dns_failure` — create route with upstream `http://this-host-absolutely-does-not-exist.invalid:8080`, trigger test → `200` with `{ status: "error", error_kind: "dns"|"connection"|"timeout" }` (accepts all three — behaviour depends on OS resolver)
  - [x] `test_connection_test_success` — spin up a `wiremock::MockServer`, register a `GET /` mock returning `200 OK`, create route pointing to mock server URL, trigger test → `200` with `{ status: "ok", http_status: 200 }`
  - [x] `test_connection_test_login_required_auth_check` — create `login_required` route, trigger test (upstream unreachable is fine) → response contains `auth_check: { configured: false }` with non-empty message

- [x] Add `testRoute` API function to admin-ui (AC: 1, 2, 3, 4)
  - [x] In `admin-ui/src/services/api.ts`: add `ConnectionTestResult` interface with fields `status`, `http_status`, `latency_ms`, `error_kind`, `error_message`, `auth_check` (nullable `AuthCheckResult` sub-interface); add `testRoute(routeId: string): Promise<ConnectionTestResult>` calling `POST /api/routes/${routeId}/test`

- [x] Add "Test" action and inline result to route rows in the admin-ui (AC: 1, 2, 3, 4)
  - [x] In `admin-ui/src/pages/ApplicationDetailPage.tsx`: add a `testResults` state `Record<string, ConnectionTestResult | 'loading'>`, add a "Test" button column header and cell to the routes table, wire button click to call `testRoute(route.id)` while setting `testResults[route.id] = 'loading'` during the call
  - [x] Render inline test result badge per row: loading → spinner text `"Testing…"`; `status === "ok"` → green badge `"✓ Reachable (HTTP {http_status})"` with optional latency; `status === "error"` → red badge with `error_message`; if `auth_check?.configured === false` → amber badge `"⚠ No IdP configured"` shown alongside

## Dev Notes

### Architecture Patterns

- The connection test is **synchronous** — make the upstream HTTP call inside the handler and return the result immediately. No background tasks or polling for this story.
- `reqwest::Client` is internally `Arc`-wrapped and cheaply `Clone`. Store one instance in `AppState` and share across handlers to reuse DNS/TLS connection pools.
- Use `reqwest::ClientBuilder::new().timeout(Duration::from_secs(5)).build()` in `AppState`. The 5s timeout applies to the entire request cycle.
- Measure elapsed time with `std::time::Instant::now()` before sending, then `.elapsed().as_millis()` after response.
- For TLS failures, `reqwest` surfaces them as connect-phase errors. Check `error.to_string().to_lowercase()` for `"tls"`, `"certificate"`, `"handshake"` substrings.
- DNS failure: `reqwest` reports these as connect-phase errors; message typically contains `"dns"` or `"failed to lookup"`.
- **Do not** follow redirects by default — configure `ClientBuilder::redirect(reqwest::redirect::Policy::limited(3))` so we detect redirect loops but don't fail on normal upstreams.

### Endpoint Convention

- `POST /api/routes/:id/test` — POST signals a mutation-like action (side-effect: outbound HTTP call). Consistent with the PRD's "run a test" framing.
- Returns `200 OK` regardless of whether the upstream was reachable — error is in the payload (`status: "error"`). Only `401` (no auth), `404` (unknown route id), or `500` (internal) differ.

### Test URL Construction

- `upstream_url` from `applications` column — e.g. `http://192.168.1.10:8080` (no trailing slash by convention)
- `path_prefix` from `routes` column — always stored with leading `/`, e.g. `/` or `/api`
- Concatenate directly: `format!("{}{}", upstream_url, path_prefix)` → `http://192.168.1.10:8080/`
- No de-duplication of slashes needed — path_prefix always `"/"` or `"/something"` (enforced by `create_route`)

### Auth Check Stub (AC 4 — Epic 3 extension point)

- In this story the auth check is intentionally a **no-op stub**: `login_required` routes always return `configured: false` with an informational message.
- Epic 3 will replace this stub with a real IdP lookup once the IdP table and model exist.
- Keep the check behind an `if route.access_mode == "login_required"` guard so `public` routes never return an `auth_check` field.

### Project Structure Notes

- `crates/control-plane/src/state.rs` currently has only `db_pool`. Adding `http_client` breaks the `AppState::new(pool)` call in all three existing test files (`auth_flow.rs`, `applications_flow.rs`, `routes_flow.rs`). Update the `create_test_app()` helper in each test file to pass a `reqwest::Client::new()`.
- `crates/shared/src/models/route.rs` — the new `get_route_with_upstream_by_id` query joins `routes r INNER JOIN applications a ON a.id = r.application_id WHERE r.id = $1`. No `enabled` filter (admin can test disabled routes). Returns `RouteWithUpstream` (struct already exists, fields: `id, host, path_prefix, access_mode, upstream_url`). Note: `access_mode` is a `String` in `RouteWithUpstream`, not the enum, matching the existing pattern.
- `wiremock = "0.6"` starts a real TCP server on a random port — works with `reqwest` client making real outbound calls. Use `wiremock::MockServer::start().await` and `wiremock::Mock::given(method("GET")).respond_with(ResponseTemplate::new(200)).mount(&mock_server).await`.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.3: Connection Test for Routes] — acceptance criteria and user story
- [Source: _bmad-output/planning-artifacts/prd.md#FR10] — functional requirement for connection test
- [Source: crates/control-plane/src/api/routes.rs] — existing route handler patterns (AppError, require_auth, path extraction)
- [Source: crates/control-plane/src/state.rs] — AppState shape; `http_client` field is new
- [Source: crates/control-plane/src/main.rs] — router wiring; add `post(api::test_route)` on the existing `/api/routes/:id` sub-path
- [Source: crates/shared/src/models/route.rs#RouteWithUpstream] — existing struct reused by new helper
- [Source: crates/gateway/Cargo.toml] — reqwest config pattern: `default-features = false, features = ["rustls-tls"]` (avoids native OpenSSL dependency)
- [Source: crates/control-plane/tests/routes_flow.rs] — integration test helper patterns (`create_test_app`, `login_and_get_cookie`, `file_serial`, `MemoryStore`)
- [Source: admin-ui/src/services/api.ts#Routes API] — existing API function patterns
- [Source: admin-ui/src/pages/ApplicationDetailPage.tsx] — routes table structure to extend with "Test" column

## Dev Agent Record

### Agent Model Used

Claude Sonnet 4.6 (GitHub Copilot)

### Debug Log References

- **Proxy bypass**: dev environment has `HTTP_PROXY` set to a corporate proxy. Added `.no_proxy()` to the test reqwest client so integration tests connect directly to localhost. Production client intentionally reads env vars (main.rs) to route traffic through any configured proxy.
- **DNS failure classification**: on this system, `.invalid` TLD queries time out (5s) rather than returning fast NXDOMAIN. The `test_connection_test_dns_failure` assertion accepts `"dns"`, `"connection"`, or `"timeout"` error kinds.

### Implementation Notes

All 7 tasks implemented as specified. Key decisions:
- `AppState::new(db_pool, http_client)` signature — required updating all three existing test files' `create_test_app()` helpers to pass `reqwest::Client::new()`.
- `classify_error()` checks `is_timeout()` first, then `is_connect()` with keyword matching (`dns`, `tls/certificate/handshake/ssl`), then falls back to generic `"connection"`.
- Auth check for `login_required` routes returns `configured: false` as specified stub; Epic 3 will replace with real IdP lookup.
- Frontend `App.test.tsx` has a pre-existing failure (missing `<Router>` wrapper) unrelated to this story.

### File List

- `crates/control-plane/Cargo.toml` — added reqwest + wiremock dependencies
- `crates/control-plane/src/state.rs` — added `http_client: reqwest::Client` field
- `crates/control-plane/src/main.rs` — wired http_client into AppState, registered POST /api/routes/:id/test
- `crates/control-plane/src/api/routes.rs` — added test_route handler, classify_error, AuthCheckResult, ConnectionTestResult
- `crates/control-plane/tests/auth_flow.rs` — updated AppState::new signature
- `crates/control-plane/tests/applications_flow.rs` — updated AppState::new signature
- `crates/control-plane/tests/routes_flow.rs` — updated AppState::new signature
- `crates/control-plane/tests/connection_test_flow.rs` — new integration tests (5 tests)
- `crates/shared/src/models/route.rs` — added get_route_with_upstream_by_id
- `admin-ui/src/services/api.ts` — added testRoute, AuthCheckResult, ConnectionTestResult
- `admin-ui/src/pages/ApplicationDetailPage.tsx` — added Test column, handleTestRoute, inline result badges
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/2-3-connection-test-for-routes.md`

### Completion Notes List

### File List
