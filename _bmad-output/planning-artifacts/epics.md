---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
inputDocuments:
  - path: _bmad-output/planning-artifacts/prd.md
    type: prd
    description: Primary Product Requirements Document
    included: true
  - path: _bmad-output/planning-artifacts/architecture.md
    type: architecture
    description: Architecture Decision Document
    included: true
  - path: _bmad-output/planning-artifacts/ux-design-specification.md
    type: ux
    description: UX Design Specification
    included: true
  - path: _bmad-output/brainstorming/brainstorming-session-2026-02-18.md
    type: brainstorming
    description: Initial product/architecture brainstorming session
    included: true
---

# skaild2 - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for skaild2, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Admin users can configure one or more external identity providers, such as Azure Entra, Google, or existing Keycloak, for use across all protected applications.
FR2: Admin users can test an identity provider configuration, for example with a test login, before enabling it for production traffic.
FR3: Admin users can define roles and groups that represent access levels used across applications.
FR4: Admin users can map identity provider attributes, such as claims and groups, to internal roles.
FR5: End users can authenticate to protected applications via single sign-on using a configured identity provider.
FR6: End users can be denied or granted access to a specific application based on their assigned roles.
FR7: Admin users can register a new application by specifying its upstream address and desired external hostname.
FR8: Admin users can define one or more routes per application, including path-based routing when needed.
FR9: Admin users can choose the access mode for a route, such as public, login required, or role-restricted.
FR10: Admin users can run a connection test for an application or route that validates upstream reachability, TLS behavior, and basic authentication integration.
FR11: Admin users can enable or disable an application or route without deleting its configuration.
FR12: The system can route HTTP and HTTPS, WebSocket, and Server-Sent Events traffic for configured applications according to defined routes.
FR13: Admin users can view and edit access policies for each application and route through the UI without editing configuration files.
FR14: Admin users can preview the effective policy for a given application or route, including which roles have access.
FR15: Admin users can review a history of configuration and policy changes with timestamps and actor information.
FR16: The system can apply configuration and policy changes without requiring a restart of the reverse proxy instances.
FR17: The system can validate configuration changes before applying them and surface validation errors to the admin user.
FR18: Admin users can view recent access activity per application, including which users accessed which routes and when.
FR19: Admin users can view counts of failed authentication or authorization attempts per application or route.
FR20: Admin users can view basic health status for each application or route, for example up, down, or degraded, based on connectivity and error rates.
FR21: Admin users can filter or search log and event views by application, route, user, or time range.
FR22: The system can export or expose logs in a format suitable for integration with external logging or monitoring tools.
FR23: Admin users can deploy the full stack on a single host using a documented Docker Compose configuration.
FR24: Admin users can configure the system to operate behind a wildcard DNS entry that fronts one or more applications.
FR25: The system can operate with multiple reverse proxy instances on the same host sharing session state via a supported session store.
FR26: Admin users can perform configuration backup and restore operations for the control plane, such as exporting and importing configuration and policies.
FR27: The system can perform safe configuration migrations when upgrading between compatible versions.
FR28: Admin users can create, update, and deactivate administrator accounts for access to the admin UI.
FR29: Admin users can configure authentication requirements for admin access, such as password policy and optional second factor when available.
FR30: Admin users can view an audit trail of sensitive admin actions, such as policy changes, identity provider changes, and application enable or disable actions.
FR31: Admin users can restrict which administrators are allowed to perform specific high-risk actions, such as managing identity providers or deleting applications.
FR32: The system can enforce secure defaults for new configurations, such as requiring authentication for new applications unless explicitly configured otherwise.

### NonFunctional Requirements

NFR1: When the reverse proxy and upstream application run on the same physical host, the additional median latency introduced by skaild2 per HTTP(S) request is under 10 ms under normal homelab workloads.
NFR2: Under sustained normal load with multiple concurrent users and long-lived WebSocket and Server-Sent Events connections, skaild2 maintains the latency target from NFR1 without becoming the primary performance bottleneck on typical homelab hardware.
NFR3: All external access to the admin interface and protected applications is served over TLS; plaintext HTTP is either redirected or disabled.
NFR4: Administrator credentials and identity provider secrets are stored using industry-standard protections, such as hashed passwords and encrypted secrets at rest.
NFR5: All authentication and authorization decisions, as well as sensitive admin actions such as policy changes, identity provider changes, and application enable or disable actions, are auditable via logs that include actor, action, target, and timestamp.
NFR6: Default configuration favors secure behavior so that new applications require authentication by default unless explicitly configured otherwise.
NFR7: The default Docker Compose deployment supports running at least two reverse proxy instances concurrently on a single host, sharing sessions via the configured session store.
NFR8: Configuration and policy changes propagate correctly to all running reverse proxy instances without requiring instance restarts and without breaking active user sessions.
NFR9: Under typical homelab traffic with at least two reverse proxy instances, skaild2 continues to meet the performance targets defined in the performance non-functional requirements.
NFR10: skaild2 exposes Prometheus-compatible metrics, for example via an HTTP `/metrics` endpoint, including at minimum request counts, error counts, authentication and authorization failures, and basic per-application traffic indicators.
NFR11: Log formats and metric labels are structured in a way that allows straightforward ingestion into common observability stacks, such as Prometheus and Grafana or Loki and ELK, without custom parsing.

### Additional Requirements

- Use a Rust backend with a Cargo workspace separating gateway, control-plane, and shared domain library.
- Use a React + TypeScript admin UI built with Vite and TailwindCSS, integrating the Mermaidcore design system.
- Use Postgres as the primary data store for configuration, policies, audit logs, and metadata.
- Separate control plane and data plane with the Rust gateway as the single public HTTP(S) entrypoint, handling TLS termination and routing.
- Store long-lived IdP refresh tokens encrypted at rest in the control-plane; only short-lived tokens are visible to the gateway.
- Provide Docker Compose–first deployment wiring gateway, control-plane, and Postgres on a private network.
- Integrate DNS and certificate automation (for example, Let’s Encrypt ACME) so that certificates are managed without manual intervention.
- Apply Mermaidcore visual language consistently across the admin UI, including dark mode, glassmorphic surfaces, and accent colors.
- Ensure operator-focused UX flows for Add Route, Connect IdP, and Issue Certificate that align with the homelab admin journey.
- Support future mapping of the same topology to Kubernetes without rethinking core architecture (compose-first, K8s-ready design).

### FR Coverage Map

FR1: Epic 1 – First-Time Deployment & Identity Setup (IdP configuration for applications)
FR2: Epic 1 – First-Time Deployment & Identity Setup (IdP configuration testing)
FR3: Epic 3 – Access Policy & Role-Based Authorization (role model definition)
FR4: Epic 3 – Access Policy & Role-Based Authorization (IdP attribute mapping)
FR5: Epic 1 / Epic 3 – First-Time Deployment & Identity Setup; Access Policy & Role-Based Authorization (SSO to applications)
FR6: Epic 3 – Access Policy & Role-Based Authorization (authorization decisions for applications)
FR7: Epic 2 – Application & Routing Management (application registration)
FR8: Epic 2 – Application & Routing Management (route definitions)
FR9: Epic 2 – Application & Routing Management (route access modes)
FR10: Epic 2 – Application & Routing Management (connection tests)
FR11: Epic 2 – Application & Routing Management (enable/disable routes and applications)
FR12: Epic 2 – Application & Routing Management (protocol support for HTTP, HTTPS, WebSockets, and SSE)
FR13: Epic 3 – Access Policy & Role-Based Authorization (policy editing UI)
FR14: Epic 3 – Access Policy & Role-Based Authorization (effective policy preview)
FR15: Epic 5 – Operations, Resilience & Admin Governance (configuration and policy change history)
FR16: Epic 5 – Operations, Resilience & Admin Governance (live application of configuration changes)
FR17: Epic 5 – Operations, Resilience & Admin Governance (configuration validation before apply)
FR18: Epic 4 – Monitoring, Audit & Troubleshooting (access activity views)
FR19: Epic 4 – Monitoring, Audit & Troubleshooting (failed auth/authorization counts)
FR20: Epic 4 – Monitoring, Audit & Troubleshooting (per-application and route health status)
FR21: Epic 4 – Monitoring, Audit & Troubleshooting (filtering and search over logs and events)
FR22: Epic 4 – Monitoring, Audit & Troubleshooting (log export and integration support)
FR23: Epic 1 / Epic 5 – First-Time Deployment & Identity Setup; Operations, Resilience & Admin Governance (Compose deployment and upgrade safety)
FR24: Epic 1 / Epic 2 – First-Time Deployment & Identity Setup; Application & Routing Management (wildcard DNS usage for applications)
FR25: Epic 5 – Operations, Resilience & Admin Governance (multi-instance session sharing)
FR26: Epic 5 – Operations, Resilience & Admin Governance (configuration backup and restore)
FR27: Epic 5 – Operations, Resilience & Admin Governance (safe configuration migrations between versions)
FR28: Epic 1 / Epic 5 – First-Time Deployment & Identity Setup; Operations, Resilience & Admin Governance (admin account lifecycle)
FR29: Epic 1 / Epic 5 – First-Time Deployment & Identity Setup; Operations, Resilience & Admin Governance (admin authentication requirements)
FR30: Epic 4 / Epic 5 – Monitoring, Audit & Troubleshooting; Operations, Resilience & Admin Governance (audit trail of sensitive admin actions)
FR31: Epic 5 – Operations, Resilience & Admin Governance (admin permission scoping for high-risk actions)
FR32: Epic 3 – Access Policy & Role-Based Authorization (secure defaults for new application configurations)

## Epic List

### Epic 1: First-Time Deployment & Identity Setup

Homelab admins can deploy skaild2 on a single host via Docker Compose, point wildcard DNS, create the first local admin account, and configure and test identity providers for use by protected applications so the system is ready to front services securely.

**FRs covered:** FR1, FR2, FR5, FR23, FR24, FR28, FR29

### Epic 2: Application & Routing Management

Admins can register upstream services, define host and path-based routes, choose access modes, and validate connectivity so traffic is correctly and safely proxied through skaild2.

**FRs covered:** FR7, FR8, FR9, FR10, FR11, FR12, FR24

### Epic 3: Access Policy & Role-Based Authorization

Admins can define roles, map identity provider attributes, and express per-application and per-route access policies so that only the right users can reach the right services with secure defaults.

**FRs covered:** FR3, FR4, FR5, FR6, FR13, FR14, FR32

### Epic 4: Monitoring, Audit & Troubleshooting

Admins can see who accessed what, detect failures, understand health, and audit sensitive admin actions so they can operate and trust skaild2 day to day.

**FRs covered:** FR18, FR19, FR20, FR21, FR22, FR30

### Epic 5: Operations, Resilience & Admin Governance

Admins can safely run and evolve skaild2 over time, including multi-instance operation, configuration backup and restore, upgrade safety, and governance over which administrators can perform high-risk actions.

**FRs covered:** FR15, FR16, FR17, FR23, FR25, FR26, FR27, FR28, FR29, FR31

## Epic 1: First-Time Deployment & Identity Setup

This epic enables a homelab admin to bring up skaild2 on a single host using Docker Compose, access the admin UI, create the first local admin account, and verify that the system is ready for application SSO by configuring and testing at least one identity provider.

### Story 1.0: Initial Project Setup from Starter Templates

As a developer working on skaild2,
I want the project scaffolded using the agreed Rust workspace and React/Vite/Tailwind/Mermaidcore stack with a base Docker Compose file,
So that all subsequent deployment and feature work builds on a consistent, reproducible foundation.

**Acceptance Criteria:**

**Given** I have a development machine with a supported OS, Rust toolchain, Node.js, Docker, and Docker Compose installed
**When** I follow the documented setup steps to clone the repository and initialize the project
**Then** the repo contains a Rust Cargo workspace with at least `gateway`, `control-plane`, and `shared` crates configured and buildable

**And Given** the workspace is initialized
**When** I open the admin UI project
**Then** there is a React + TypeScript app created with Vite, wired to use TailwindCSS and the Mermaidcore design tokens from the UX specification

**And Given** the backend services and admin UI projects exist
**When** I open the provided `docker-compose.yml`
**Then** it defines services for the gateway, control-plane, and Postgres (and any required supporting services) on a private network, suitable as the baseline for later deployment stories

**And Given** the initial setup has completed successfully
**When** I run the documented commands to build the Rust services and start the dev stack (locally or via Compose)
**Then** the code builds without errors and the containers start to the point where subsequent deployment and configuration stories can proceed

### Story 1.1: Single-Host Compose Deployment with First Admin Login

As a homelab admin,
I want to deploy skaild2 on a single server with Docker Compose and sign in to the admin UI with a local admin account,
So that I have a running control plane ready to configure identity providers and applications.

**Acceptance Criteria:**

**Given** I have a Linux host with Docker and Docker Compose installed
**When** I clone the skaild2 repository and run the documented `docker compose up` command
**Then** the stack starts successfully and exposes the admin UI on the expected hostname or port

**And Given** the admin UI is reachable in a browser
**When** I complete the initial setup flow to create the first local admin account
**Then** I can sign in to the admin UI using that account

**And Given** the stack is running from the Compose setup
**When** I stop and restart the Docker Compose stack using the documented commands
**Then** the admin UI is reachable again and the previously created admin account still works

## Epic 2: Application & Routing Management

This epic allows a homelab admin to register upstream applications, define host and path-based routes, choose access modes, and validate connectivity so that traffic flows safely and correctly through skaild2.

### Story 2.1: Register First Upstream Application

As a homelab admin,
I want to register an upstream HTTP service and bind it to a hostname under my wildcard domain,
So that skaild2 knows where to send proxied traffic for that application.

**Acceptance Criteria:**

**Given** I am signed in as an admin
**When** I navigate to the Applications section in the admin UI and choose to add a new application
**Then** I can enter at least an application name, an upstream URL (including scheme and port), and a desired external hostname under the configured wildcard domain

**And Given** I complete the form with a valid upstream URL and hostname
**When** I save the new application
**Then** the application is persisted in the control-plane database and appears in the Applications list with its configured upstream and hostname

**And Given** I have registered at least one application
**When** I return to the Applications list later or after restarting the stack
**Then** the application still appears with the same upstream and hostname configuration

**And Given** I have an existing application registered
**When** I edit its upstream URL or hostname using the admin UI
**Then** the updated values are saved and reflected consistently wherever the application is shown

### Story 2.2: Define Basic Route and Access Mode

As a homelab admin,
I want to define at least one route for an application and choose whether it requires login or is public,
So that I can control how users reach the upstream service through skaild2.

**Acceptance Criteria:**

**Given** at least one application is registered
**When** I open that application’s detail view and choose to add a route
**Then** I can specify at minimum a host (pre-filled from the application), an optional path prefix, and an access mode (public vs login-required)

**And Given** I create a login-required route for the application
**When** I save the route configuration
**Then** the route is persisted and appears in the application’s route list with its access mode clearly indicated

**And Given** the route exists and the gateway is running
**When** I browse to the route’s external URL in a browser as an unauthenticated user
**Then** I am redirected to the configured identity provider login flow before the upstream application is reached

**And Given** I have successfully authenticated via the identity provider
**When** I am redirected back to the route
**Then** the request is proxied to the configured upstream service and the application loads as expected

**And Given** a public route exists for an application
**When** I browse to that route’s external URL
**Then** I can reach the upstream service without being forced through the identity provider login flow

### Story 2.3: Connection Test for Routes

As a homelab admin,
I want to run a connection test for a route,
So that I can quickly see whether upstream connectivity, TLS, and basic auth wiring are correct before exposing the route broadly.

**Acceptance Criteria:**

**Given** at least one route exists for a registered application
**When** I trigger a connection test from the route’s detail view or actions menu
**Then** the system attempts to contact the configured upstream using the stored URL and reports success or failure clearly in the UI

**And Given** the upstream is reachable and returns a successful response
**When** the connection test completes
**Then** I see a success status that indicates the route is reachable and ready for traffic

**And Given** the upstream URL is invalid, the host cannot be resolved, or TLS handshakes fail
**When** the connection test runs
**Then** I see a failed status along with a short, actionable error description (for example, “DNS resolution failed”, “TLS certificate validation failed”, or “connection timed out”)

**And Given** the route requires login through an identity provider
**When** I run the connection test
**Then** the test verifies that the auth flow for that route is configured (for example, required IdP is set) and surfaces a clear error if mandatory auth configuration is missing

## Epic 3: Access Policy & Role-Based Authorization

This epic enables admins to define global roles, map identity provider attributes, and apply per-application and per-route policies with secure defaults so only the right users reach the right services.

### Story 3.1: Define Global Roles and Map IdP Claims

As a homelab admin,
I want to define global roles and map IdP claims or groups into those roles,
So that I can express access policies in terms of a consistent role model across applications.

**Acceptance Criteria:**

**Given** I am signed in as an admin
**When** I navigate to the Roles or Access Policy section in the admin UI
**Then** I can create, rename, and delete global roles (for example, `admin`, `family`, `friends`) that are stored in the control-plane database

**And Given** at least one identity provider is configured
**When** I edit role mappings
**Then** I can define rules that map IdP claims or groups (for example, `group=homelab-admins`) to one or more global roles

**And Given** I have saved mappings from IdP claims to roles
**When** a user successfully signs in through the identity provider
**Then** the system resolves their effective roles based on those mappings and persists or caches them for use in authorization decisions

**And Given** I remove or change a role mapping
**When** the same user signs in again
**Then** their effective roles reflect the updated mappings

### Story 3.2: Require Role-Based Access for a Route

As a homelab admin,
I want to restrict a route so that only users with specific roles can access it,
So that sensitive applications are protected beyond simple login-required access.

**Acceptance Criteria:**

**Given** at least one role and at least one login-required route exist
**When** I edit the route’s access policy
**Then** I can select one or more roles that are required to access that route

**And Given** a user signs in successfully through the identity provider but does not have any of the required roles
**When** they attempt to access the protected route
**Then** the gateway denies access and returns a clear error page or status indicating they are not authorized

**And Given** another user signs in successfully and does have at least one of the required roles
**When** they access the same route
**Then** their request is proxied to the upstream application and succeeds

**And Given** I later remove a role from the route’s access policy
When** an affected user accesses the route again
**Then** authorization decisions reflect the updated policy without requiring a restart of the gateway

### Story 3.3: Policy Preview for a Route and Secure Defaults

As a homelab admin,
I want to see a clear summary of the effective access policy for each route,
So that I can quickly understand who can reach a given application and verify secure defaults.

**Acceptance Criteria:**

**Given** at least one route exists
**When** I open the route’s detail view in the admin UI
**Then** I see a policy summary that shows whether the route is public, login-required, or role-restricted, and which roles and IdPs (where applicable) are in effect

**And Given** no explicit access policy has been set for a newly created application or route
**When** I view its policy summary
**Then** it indicates a secure default (for example, login-required) in line with the PRD’s secure-default requirement

**And Given** I change a route from public to login-required or role-restricted
**When** I refresh the route’s policy summary
**Then** the summary reflects the updated access mode and required roles without ambiguity

**And Given** a configuration change introduces an invalid or incomplete policy (for example, role-based mode with no roles selected)
**When** I view the route’s policy summary
**Then** I see a clear warning and guidance on how to fix the configuration before it can be safely used

## Epic 4: Monitoring, Audit & Troubleshooting

This epic gives admins visibility into who accessed what, where failures occur, and basic health so they can operate and trust skaild2 day to day.

### Story 4.1: Per-Application Access Activity View

As a homelab admin,
I want to see recent access events per application,
So that I can understand who is using each service and when.

**Acceptance Criteria:**

**Given** users have accessed protected routes through skaild2
**When** I open an application’s Access Activity view in the admin UI
**Then** I see a time-ordered list of recent requests including at least timestamp, user identity (or anonymous), route, and outcome (success/denied)

**And Given** there are many events
**When** I filter or page through the access activity
**Then** I can narrow results by time window and optionally by route or user identity

**And Given** no requests have been made recently for an application
**When** I open its Access Activity view
**Then** I see an empty state that clearly indicates there is no recent access rather than an error

### Story 4.2: Failed Auth/Authorization Overview

As a homelab admin,
I want to see a summary of failed authentication and authorization attempts,
So that I can detect misconfigurations and possible abuse.

**Acceptance Criteria:**

**Given** there have been failed login or authorization attempts
**When** I open a Failed Access or Security view in the admin UI
**Then** I see aggregated counts of failed authentication and authorization attempts per application and/or route over a selectable time window

**And Given** I click into a specific application or route from that view
**When** I drill down
**Then** I can see representative recent failed events with enough context (timestamp, user identity or IdP subject where available, failure reason) to understand what is going wrong

**And Given** there have been no recent failures
**When** I look at the failure overview
**Then** it clearly indicates zero failures rather than appearing broken or empty without explanation

### Story 4.3: Basic Health and Log Export for Integration

As a homelab admin,
I want to see basic health status for applications and export logs/metrics in a structured way,
So that I can integrate with external observability tools when needed.

**Acceptance Criteria:**

**Given** at least one application and its routes are configured
**When** I open a Health or Status view in the admin UI
**Then** I see per-application and per-route status (for example, up, down, degraded) based on recent connectivity and error rates

**And Given** I have access to the skaild2 metrics endpoint
**When** I query the Prometheus-compatible `/metrics` endpoint from a browser or Prometheus instance
**Then** I see structured metrics including at least request counts, error counts, and auth/authorization failures labeled by application and route

**And Given** I configure my external logging or metrics stack to ingest skaild2 logs/metrics
**When** I point it at the appropriate endpoints or log streams
**Then** the structured formats and labels allow me to build basic dashboards without custom parsing beyond configuration

## Epic 5: Operations, Resilience & Admin Governance

This epic ensures that skaild2 can be run safely over time, including multi-instance operation on a single host, configuration backup and restore, safe upgrades, and governance over admin capabilities.

### Story 5.1: Multi-Instance Proxy with Shared Sessions on Single Host

As a homelab admin,
I want to run multiple gateway instances on a single host sharing session state,
So that I can improve resilience and handle more concurrent connections without breaking logins.

**Acceptance Criteria:**

**Given** the default Docker Compose stack is running
**When** I enable or add a second gateway container as documented
**Then** both gateway instances start successfully and connect to the shared session store (for example, Redis or equivalent)

**And Given** a user has logged in through one gateway instance
**When** subsequent requests for that user are routed (by Docker or an external load balancer) to the other gateway instance
**Then** their session remains valid and they are not forced to re-authenticate solely because the instance changed

**And Given** I temporarily stop one gateway instance
**When** traffic continues through the remaining instance
**Then** existing authenticated sessions remain valid and new logins continue to work

### Story 5.2: Configuration Backup and Restore

As a homelab admin,
I want to back up and restore the control-plane configuration,
So that I can recover from mistakes or host failures without manually recreating all settings.

**Acceptance Criteria:**

**Given** applications, routes, IdPs, roles, and policies are configured
**When** I use a Backup/Export function in the admin UI or a documented CLI
**Then** I can produce an export artifact (for example, an encrypted file or snapshot) that captures the current configuration and policies

**And Given** I have a compatible backup artifact
**When** I restore it into a clean or freshly installed skaild2 instance using the documented Restore/Import function
**Then** the applications, routes, IdPs, roles, and policies are reinstated without manual re-entry

**And Given** a restore is attempted with an incompatible or corrupted artifact
**When** the restore process runs
**Then** it fails safely with a clear error message and does not partially apply a broken configuration

### Story 5.3: Basic Admin Audit Trail and Governance

As a homelab admin responsible for security,
I want a basic audit trail of sensitive admin actions and a way to scope which admins can perform high-risk operations,
So that I can understand who changed what and prevent accidental or malicious misuse.

**Acceptance Criteria:**

**Given** there are one or more admin accounts
**When** an admin performs sensitive actions such as creating or deleting applications, changing IdP configurations, or editing policies
**Then** each action is recorded in an audit log with at least timestamp, admin identity, action type, and target

**And Given** I open the Admin Audit view in the UI
**When** I filter or browse
**Then** I can see recent admin actions and search or filter by admin identity, action type, and time window

**And Given** there are different classes of admin (for example, full admins vs read-only or limited admins)
**When** I configure which roles or admin accounts are allowed to perform high-risk actions (such as managing IdPs or deleting applications)
**Then** the system enforces these constraints so that admins without sufficient permission cannot execute those actions