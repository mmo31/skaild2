---
stepsCompleted:
  - step-01-init
  - step-02-context
  - step-03-starter
inputDocuments:
  - path: _bmad-output/planning-artifacts/prd.md
    type: prd
    description: Primary Product Requirements Document
    included: true
  - path: _bmad-output/brainstorming/brainstorming-session-2026-02-18.md
    type: brainstorming
    description: Initial product/architecture brainstorming session
    included: true
workflowType: 'architecture'
project_name: 'skaild2'
user_name: 'mathieu'
date: '2026-02-18'
---

# Architecture Decision Document

## Project Context Analysis
Project Context Analysis
Requirements Overview
Functional Requirements:

Provide a self-hosted, identity-aware reverse proxy that sits in front of applications and services.
Offer a primarily UI-driven configuration experience via an admin dashboard (Dashboard, Routes, Identity, Certificates, Settings).
Automate subdomain management for routed services.
Automate SSL/TLS certificate issuance and renewal (e.g., via Let’s Encrypt) for those subdomains.
Integrate with major identity providers using modern SSO protocols, including:
OIDC providers
Azure Entra
Google
Support role-based authorization policies that can be applied per route.
Expose clear status and health information for proxies, certificates, SSO integrations, and active sessions.
Provide safe, guided flows for key actions:
“Add Route”
“Connect IdP”
“Issue Certificate”
Non-Functional Requirements:

Security:
Strong default security posture for identity, transport (TLS), and admin access.
Correct and safe handling of OIDC / SSO flows (no token leakage, secure redirects, etc.).
Separation of concerns between control plane (admin UI + config) and data plane (traffic proxy), with secure communication between them.
Performance:
Low-latency request handling for proxied traffic.
Efficient certificate and DNS automation that does not block or degrade normal traffic.
Operability:
Simple installation and self-hosting story suitable for homelab and smaller teams, with a path to more advanced setups.
Clear, observable system state (routes, certificates, IdP connections, authorization rules).
UX:
“Ultra-easy” configuration with opinionated defaults and minimal required inputs.
Consistent visual language (Mermaidcore) with high contrast, clear affordances, and accessible dark-mode design.
Scale & Complexity:

Primary domain: admin web UI + reverse proxy + identity integration.
Complexity level: medium (driven mainly by security, identity, and infra integration).
Estimated architectural components:
Admin web frontend
Control-plane API / configuration service
Proxy / gateway (data plane)
Identity / auth integration layer
Certificate/DNS automation agent
Persistence/config storage (Postgres)
Technical Constraints & Dependencies
Self-hosted deployment model; must run in user-controlled environments (homelab, small org, possibly enterprise).
Tight coupling to:
DNS and certificate automation (Let’s Encrypt ACME, DNS APIs).
External identity providers (OIDC, Azure Entra, Google).
Need for secure handling of secrets and tokens for IdPs and certificate automation.
Admin UI and UX must align with the Mermaidcore design system (Tailwind/utility-first styling, dark mode, glassmorphic surfaces).
Cross-Cutting Concerns Identified
Identity and Access Management:
Authentication via external IdPs.
Role- and policy-based authorization applied to routes.
Security:
End-to-end TLS, secure cookie/session management, hardened admin surface.
Configuration & State Management:
Single source of truth for routes, policies, IdP connections, and certificates.
Safe, auditable changes and rollbacks.
Observability:
Visibility into proxy health, certificate status, IdP connectivity, and access events.
UX Consistency:
Applying Mermaidcore design tokens and components consistently across the admin UI.
Starter Template Evaluation
Primary Technology Domain
Full-stack web application with:

Rust backend (control plane + identity-aware reverse proxy)
React + TypeScript admin UI
Postgres as primary data store
Docker Compose–first deployment, with Kubernetes as a later expansion path
Starter Options Considered
Admin UI

Vite + React + TypeScript starter
Modern, fast dev server and build pipeline
First-class TypeScript support
Easy integration with TailwindCSS and Mermaidcore design tokens
Minimal framework lock-in; suitable for an operator-focused admin dashboard
Backend

Custom Rust Cargo workspace
Async runtime (Tokio) with a web framework such as axum or actix-web
Separate binaries for:
gateway (reverse proxy, request routing, auth enforcement)
control-plane (admin API, background workers for DNS/ACME/IdP sync)
Shared library crate for domain models, config schemas, and data access (Postgres)
No dependency on Nginx, Traefik, or other external reverse proxies; the Rust gateway is the edge component
Selected Starter: Vite + React Admin + Rust Workspace Backend
Rationale for Selection:

Matches your preference for Rust and React while keeping the stack focused and composable.
Vite + React + TypeScript provides a lightweight but powerful foundation for the Mermaidcore admin UI.
A Rust workspace cleanly separates gateway and control-plane responsibilities while sharing types and persistence code.
Postgres fits well for configuration, policies, and audit logs.
Docker Compose is straightforward to wire:
gateway container
control-plane container
postgres container
optional supporting services (e.g., DNS/ACME helper later)
Avoids hard coupling to Nginx/Traefik; the gateway service exposes HTTP(S) directly and owns TLS and routing behavior.
Initialization Commands (Conceptual):

Architectural Decisions Provided by Starter

Language & Runtime

Rust for backend services, targeting async I/O with Tokio.
TypeScript for the React admin UI.
Styling Solution

Vite stack compatible with TailwindCSS; admin UI will integrate Mermaidcore tokens and components.
Build Tooling

Vite for frontend dev/build.
Cargo for Rust compilation and workspace management.
Docker for containerization, orchestrated via Docker Compose.
Testing Framework

Frontend: Jest/Vitest + Testing Library (to be detailed in implementation).
Backend: Rust’s built-in test framework with integration tests hitting gateway/control-plane endpoints.
Code Organization

gateway: reverse proxy, auth enforcement, request path.
control-plane: configuration APIs, background tasks.
shared: domain types, DB access, config parsing.
React admin is a separate project consuming the control-plane API.
Development Experience

Hot-reload dev server for the admin UI via Vite.
Fast incremental Rust builds via Cargo.
Local development via Docker Compose:
single docker-compose.yml standing up Postgres, gateway, control-plane, and optionally admin UI.
Architecture Principles
Self-Sufficient Edge Gateway

The Rust gateway is the single public HTTP(S) entrypoint; no Nginx/Traefik dependency.
TLS termination, routing, and per-request auth are all handled at the edge.
Clear Control-Plane / Data-Plane Separation

Data plane: gateway handles live traffic and enforces policies.
Control plane: control-plane manages configuration, IdP integration, DNS/ACME orchestration, and background jobs.
Global IdPs, Per-Route Policies

IdPs (OIDC, Entra, Google) configured globally in the control-plane.
Each route declares its own auth policy and references one or more global IdPs.
Role model:
Global role namespace (e.g., admin, ops, viewer).
Claims-mapping from each IdP into those global roles.
Route policies primarily reference global roles, with an advanced option for idp:role pairs.
Token Handling & Security Posture

Gateway only holds short-lived tokens needed to process requests.
Control-plane stores long-lived refresh tokens (encrypted at rest) for IdPs and uses them for session and metadata management; gateway never sees refresh tokens.
Strong defaults: TLS everywhere, secure cookies, hardened admin surface.
Compose-First, K8s-Ready

Default deployment is Docker Compose with a small number of services.
The same topology can be mapped to Kubernetes as a next phase without rethinking the core architecture.
Single Source of Truth

Postgres as the canonical store for routes, IdPs, roles, policies, and cert metadata.
Configuration changes are versioned/auditable via the control-plane, not edited ad hoc on the gateway.
Operator-Centric UX

Admin UI expresses the architecture concepts in simple flows (Add Route, Connect IdP, Issue Certificate).
Mermaidcore visual language reinforces clarity and hierarchy instead of distracting.
Core Architectural Decisions
Decision Priority Analysis
Critical Decisions (block implementation):

Stack: Rust backend + React/TS admin + Postgres.
Topology: gateway (edge), control-plane, postgres, all on a private network.
Auth pattern:
Global IdPs; per-route policies selecting IdPs.
Global role namespace with claims mapping.
Short-lived tokens at gateway; refresh tokens owned by control-plane.
Deployment baseline: Docker Compose with gateway as only published HTTP(S) service.
Important Decisions (shape architecture):

Cargo workspace split (gateway, control-plane, shared).
Vite-based frontend starter and Tailwind/Mermaidcore design system.
TLS termination and ACME handling at the gateway.
Internal-only Postgres (no direct host exposure).
Deferred Decisions (post-MVP):

K8s deployment manifests and Helm charts.
Pluggable DNS providers and ACME DNS-01 support.
Observability stack (Prometheus/Grafana/Loki, etc.).
Data Architecture
Database: Postgres 16 (target) as primary relational store.
Core entities (high level):
idp: global IdP configs (client IDs, secrets, endpoints, claims-mapping rules).
role: global role definitions.
user_binding (or equivalent): mapping between IdP subject/claims and global roles.
route: route definitions (host, path, upstream target, TLS settings).
route_policy: per-route auth/role policies referencing global roles and IdPs.
certificate: cert + key metadata, ACME account and challenge state.
Migrations: managed by a Rust-based migrator (or embedded migration system) run via the migrator container or at control-plane startup.
Authentication & Security
Auth method: browser-based SSO via external IdPs using OIDC/OAuth2.
IdP model:
Global IdP configurations stored in control-plane.
Each route can select one or more IdPs from that pool.
Roles:
Global role namespace; claims-mapping from IdP-specific groups/claims into global roles.
Route policies expressed primarily against global roles.
Tokens:
Gateway: consumes ID/access tokens and session cookies.
Control-plane: stores and rotates refresh tokens (encrypted) and manages IdP sessions.
Edge security:
TLS termination at gateway.
Mutual TLS or token-based auth between gateway and control-plane if needed later.
API & Communication Patterns
Control-plane API:
REST/JSON over HTTP for admin and configuration operations.
Authenticated via session tied to SSO login.
Gateway–Control-plane communication:
Internal HTTP API for config fetch, policy updates, and ACME/DNS operations.
External traffic:
Gateway proxies HTTP(S) requests to upstream services on the internal network.
Frontend Architecture
Framework: React + TypeScript via Vite starter.
Routing: SPA routing for the admin dashboard (e.g., /routes, /identity, /certificates, /settings).
State management:
Local component state + query/mutation layer (e.g., React Query) for control-plane APIs.
Styling:
TailwindCSS + Mermaidcore tokens/components for consistent dark, glassmorphic UI.
Infrastructure & Deployment
Docker Compose (see docker-compose.yml):

gateway:

Image: Rust binary handling TLS, routing, and auth.
Ports: 80:80, 443:443 to host.
Volume: gateway-certs for certificates and keys.
Env: points to control-plane and postgres via internal network.
control-plane:

Internal-only HTTP API (no host ports).
Talks to postgres and IdPs; holds encrypted secrets and refresh tokens.
postgres:

Internal-only; data in pgdata volume.
migrator (optional):

One-shot container running DB migrations.
Networking:

Single skaild2-net bridge network.
Only gateway is exposed to the host/internet.
Kubernetes (later):

Mirror the same topology with Deployments/Services/PVCs, with gateway as the only public Service.
System Architecture Overview (Diagram)
graph TD
  browser[Admin / User Browser] -->|HTTPS + SSO| gw[Gateway (Rust)]
  gw -->|Admin API| cp[Control-plane API (Rust)]
  gw -->|Proxy traffic| upstream[Upstream Apps / Services]

  cp --> db[(Postgres)]
  cp --> idps[IdPs: OIDC / Entra / Google]

  subgraph Data Plane
    gw
  end

  subgraph Control Plane
    cp
    db
  end

Deployment Topology (Docker Compose) – Diagram
graph LR
  internet[(Internet)] -->|80/443| gw[Gateway Container]

  subgraph skaild2-net
    gw --> cp[Control-plane Container]
    cp --> db[(Postgres Container)]
  end

  gw -. volume .- certs[(gateway-certs)]
  db -. volume .- pgdata[(pgdata)]