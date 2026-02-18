---
project_name: skaild2
created_at: 2026-02-18
source_prd: _bmad-output/planning-artifacts/prd.md
source_epics: _bmad-output/planning-artifacts/epics.md
---

# skaild2 – Project Plan

## 1. Delivery Overview

Goal: Deliver a self-hosted, identity-aware reverse proxy with a UI-first admin experience, starting with a single-host Docker Compose deployment and a minimal but solid control plane and data plane.

Initial implementation is organized around five user-value epics, derived from the PRD and epics breakdown. Execution assumes a solo/full‑stack developer with ops skills and short, focused implementation slices.

## 2. Epics and High-Level Sequence

1. **Epic 1 – First-Time Deployment & Identity Setup**  
   Bring up skaild2 on a single host via Docker Compose, expose the admin UI, create local admin accounts, and configure/test identity providers for use by applications.

2. **Epic 2 – Application & Routing Management**  
   Register upstream services, define host/path routes, choose access modes, and validate connectivity so traffic flows correctly through the proxy.

3. **Epic 3 – Access Policy & Role-Based Authorization**  
   Define roles, map IdP attributes, and express per-app/per-route policies with secure defaults.

4. **Epic 4 – Monitoring, Audit & Troubleshooting**  
   Provide access, health, and audit views so admins can see who accessed what and diagnose issues.

5. **Epic 5 – Operations, Resilience & Admin Governance**  
   Add multi-instance operation on a single host, configuration backup/restore, safe migrations, and tighter admin governance.

Recommended build order: Epic 1 → Epic 2 → Epic 3 → Epic 4 → Epic 5, with some stories in Epics 4 and 5 interleaved as needed for observability and safety.

## 3. Initial Story Plan (MVP Slice)

This section highlights the first set of stories to reach a usable MVP aligned with the PRD.

### 3.1 Epic 1 – First-Time Deployment & Identity Setup

- **Story 1.0: Initial Project Setup from Starter Templates**  
  Initialize the Rust Cargo workspace (gateway, control-plane, shared), scaffold the React/Vite/Tailwind/Mermaidcore admin UI, and create the baseline Docker Compose file wiring gateway, control-plane, and Postgres on a private network.

- **Story 1.1: Single-Host Compose Deployment with First Admin Login**  
  Deploy the stack via Docker Compose, expose the admin UI, create the first local admin, and verify that the admin user can log back in after restart.

- **Story 1.2: Identity Provider Configuration and Test (Azure Entra or Google)**  
  Add UI and control-plane support to register an external IdP (Azure Entra or Google), store credentials securely, and run a test login flow that confirms tokens can be obtained and validated.

- **Story 1.3: Wildcard DNS & Base Health Check**  
  Document and verify wildcard DNS expectations, and expose a basic health/status endpoint or page confirming that gateway, control-plane, and database are healthy.

### 3.2 Epic 2 – Application & Routing Management

- **Story 2.1: Register First Upstream Application**  
  Allow an admin to register an upstream HTTP service and bind it to a chosen hostname under the wildcard domain.

- **Story 2.2: Define Basic Route and Access Mode**  
  Support at least one route per app with a chosen access mode (public vs login-required) and ensure requests are proxied correctly.

- **Story 2.3: Connection Test for Routes**  
  Provide a connection test that checks upstream reachability, TLS behavior, and basic auth wiring for a route, surfacing clear errors in the UI.

### 3.3 Epic 3 – Access Policy & Role-Based Authorization

- **Story 3.1: Define Global Roles and Map IdP Claims**  
  Allow admins to define roles and map IdP attributes (groups/claims) to those roles.

- **Story 3.2: Require Role-Based Access for a Route**  
  Let admins configure a route so that only users with specific roles can access it, and enforce this in the gateway.

- **Story 3.3: Policy Preview for a Route**  
  Provide a UI view that summarizes the effective policy for a route (which roles and IdPs are allowed).

### 3.4 Epic 4 – Monitoring, Audit & Troubleshooting (MVP scope)

- **Story 4.1: Per-Application Access Activity View**  
  Show recent access events per application, including which user accessed which route and when.

- **Story 4.2: Failed Auth/Authorization Overview**  
  Track and display failed login/authorization attempts per application or route.

### 3.5 Epic 5 – Operations, Resilience & Admin Governance (MVP scope)

- **Story 5.1: Multi-Instance Proxy with Shared Sessions on Single Host**  
  Run at least two gateway instances via Compose, sharing session state via Redis or equivalent.

- **Story 5.2: Configuration Backup and Restore**  
  Allow exporting and importing control-plane configuration and policies.

- **Story 5.3: Basic Admin Audit Trail**  
  Log and expose a minimal audit trail of admin actions such as policy changes and IdP configuration updates.

## 4. Suggested Execution Phases (Sprint-Oriented)

Assume 1–2 week iterations with you as a solo dev; each “sprint” below is a logical grouping rather than a hard timebox.

### Sprint 1 – Bootstrap & Control Plane Foundation

- Story 1.0: Initial Project Setup from Starter Templates  
  - Outcome: Rust workspace, admin UI project, and baseline Docker Compose are all in place and build/run successfully, matching the architecture decisions.
- Story 1.1: Single-Host Compose Deployment with First Admin Login  
  - Outcome: `docker compose up` brings up gateway, control-plane, DB, and admin UI; first local admin can log in after restart.

### Sprint 2 – IdP Wiring & Basic Health

- Story 1.2: Identity Provider Configuration and Test (Azure Entra or Google)  
  - Outcome: admin can register an IdP, store secrets securely, and run a test login that returns a validated token.
- Story 1.3: Wildcard DNS & Base Health Check  
  - Outcome: documented wildcard DNS pattern; /health or equivalent shows gateway, control-plane, and DB status.
- Supporting tasks:
  - Implement minimal IdP config persistence model in control-plane.
  - Add basic health endpoints for gateway and control-plane.

### Sprint 3 – First Application & Routing

- Story 2.1: Register First Upstream Application  
  - Outcome: admin can define an upstream service and external hostname.
- Story 2.2: Define Basic Route and Access Mode  
  - Outcome: at least one login-required route proxies correctly to the upstream.
- Story 2.3: Connection Test for Routes  
  - Outcome: route test clearly reports connectivity/TLS/auth wiring issues.

### Sprint 4 – Roles & Policy Basics

- Story 3.1: Define Global Roles and Map IdP Claims  
  - Outcome: global roles exist; IdP claims mapping persisted and applied at login.
- Story 3.2: Require Role-Based Access for a Route  
  - Outcome: route can be locked down to specific roles; unauthorized users are blocked.
- Story 3.3: Policy Preview for a Route  
  - Outcome: admin UI shows effective policy (roles/IdPs) for a route.

### Sprint 5 – Monitoring & Minimal Audit

- Story 4.1: Per-Application Access Activity View  
  - Outcome: recent access events per app visible to admins.
- Story 4.2: Failed Auth/Authorization Overview  
  - Outcome: failed login/auth counts per app/route surfaced.
- Story 5.3: Basic Admin Audit Trail  
  - Outcome: sensitive admin actions (IdP changes, policy edits) logged and viewable.

### Sprint 6 – Resilience & Operations

- Story 5.1: Multi-Instance Proxy with Shared Sessions on Single Host  
  - Outcome: two gateway instances running via Compose, sharing sessions via Redis or equivalent.
- Story 5.2: Configuration Backup and Restore  
  - Outcome: admin can export/import configuration and policies to recover from issues.

## 5. Next Steps

- Use this sprint outline as the basis for more detailed sprint-planning docs (for example, using the bmad sprint-planning workflows if desired).
- For the upcoming sprint, take the stories listed under the chosen sprint and expand acceptance criteria and technical notes directly in [planning artifacts/epics.md](_bmad-output/planning-artifacts/epics.md).
