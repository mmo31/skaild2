---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-02b-vision
  - step-01b-continue
  - step-02c-executive-summary
  - step-03-success
  - step-04-journeys
  - step-05-domain
  - step-06-innovation
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
  - step-08-scoping
inputDocuments:
  - path: _bmad-output/brainstorming/brainstorming-session-2026-02-18.md
    type: brainstorming
    description: Initial product/architecture brainstorming session
    included: true
documentCounts:
  briefs: 0
  research: 0
  brainstorming: 1
  projectDocs: 0
workflowType: 'prd'
project_name: skaild2
user_name: mathieu
communication_language: English
user_skill_level: intermediate
output_folder: _bmad-output
planning_artifacts: _bmad-output/planning-artifacts
project_knowledge: docs
classification:
  projectType: identity-aware reverse proxy with admin web UI
  domain: identity and access management / security tooling for homelab and enterprise SSO
  complexity: medium
  projectContext: greenfield
---

# Product Requirements Document - skaild2

**Author:** mathieu
**Date:** 2026-02-18

## Executive Summary

This project delivers an identity-aware reverse proxy for home lab and small self-hosted environments that want enterprise-grade security, reliability, and observability without enterprise-grade complexity. It provides a UI-first experience for configuring routes, identity providers, and authorization policies, so administrators can safely manage secure ingress without editing YAML or restarting services. The system automates HTTPS (via Let's Encrypt), subdomain management, and SSO (OIDC, including providers like Azure Entra and Google), while remaining lightweight and performant enough to run comfortably on typical homelab hardware.

The product targets technically capable home lab operators and self-hosters who care deeply about security and correctness but don’t have time to hand-craft Nginx, Traefik, or Pomerium setups. The goal is to make “enterprise-style” protected access (zero-trust-ish front door, centralized identity, role-aware routing) feel as simple as managing an app dashboard: define services, connect IdPs, assign roles, and rely on the system to enforce policies consistently.

### What Makes This Special

Compared to existing reverse proxies and identity-aware gateways, this product combines Pomerium-class capabilities with a significantly better operational experience. Configuration is fully UI-based and hot-reloaded — admins can add or change routes, identity providers, and authorization rules without service restarts or manual config deployments. Horizontal scaling and stateless design make it straightforward to add instances for resilience and throughput, while maintaining a single source of truth for configuration.

Security and observability are first-class: the product offers opinionated defaults, clear audit trails, and easy monitoring/log analysis so admins can see who accessed what, when, and under which role. Authorization is modeled with rich, role-based policies that can be expressed and reviewed in the UI, making it much easier to reason about and evolve access rules over time. The result is a tool that gives home lab operators “enterprise-grade” control and insight without forcing them to live inside complex config files.

## Project Classification

- Project type: identity-aware reverse proxy with admin web UI  
- Domain: identity and access management / security tooling for homelab and SSO-style protected services  
- Complexity: medium — substantial security and auth surface area, but focused scope around reverse proxying and access control  
- Project context: greenfield implementation with architecture and UX designed from scratch for a UI-driven, horizontally scalable control plane

## Success Criteria

### User Success

- A technically capable home lab admin can deploy the system (including TLS, IdP wiring, and at least one protected app) in under one hour using a single, documented path (for example, Docker Compose or a Helm chart), without hand-editing reverse-proxy configuration files.
- Common day-2 operations (adding a new service, changing an authorization rule, rotating secrets) are performed entirely via the UI and feel safe enough that the admin does not fear breaking existing access.
- Power users feel confident enough in the product to recommend it to peers as their default front door for self-hosted services.

### Business Success

- Within 3–6 months of first usable release, typical users are fronting a meaningful share of their exposed services (for example, core homelab apps) through this proxy rather than ad hoc reverse-proxy configurations.
- A visible minority of users (power users) actively recommend the product in relevant communities (homelab, self-hosting, security) as the easiest way to get enterprise-grade SSO and authorization at home.
- User feedback focuses primarily on feature gaps and improvements, not on deployment or operational complexity.

### Technical Success

- The reverse proxy adds minimal overhead: low additional latency under normal homelab loads and stable throughput on modest hardware, such that performance is not perceived as a bottleneck compared to existing setups.
- The system fully supports modern long-lived and streaming use cases required in homelab environments, including WebSockets, Server-Sent Events (SSE), and common HTTP API and dashboard scenarios.
- Security posture is at least on par with tools like Pomerium, with hardened defaults, correct handling of authentication and authorization, and no critical misconfiguration-by-default risks.

### Measurable Outcomes

- Time to first protected service from a clean environment fits within a single working session for a typical power user, with a target well under one hour.
- A typical user can add or modify a service and its authorization rules entirely from the UI without downtime or container restarts.
- All officially supported deployment paths (for example, Docker Compose and Kubernetes Helm charts) support WebSockets and SSE out of the box with no special manual tuning.
- User satisfaction, measured through informal feedback or small surveys, indicates that deployment and day-2 operations are simpler than their previous do-it-yourself reverse-proxy approach.

## Product Scope

### MVP – Minimum Viable Product

- Single-node deployment path (for example, Docker Compose or a simple Helm chart) with clear, opinionated setup for TLS, identity provider integration, and at least one protected application.
- Core identity-aware reverse proxy features: HTTPS routing, OIDC integration, role-based authorization, and a UI for configuring services, identity providers, and authorization policies.
- Native support for WebSockets and Server-Sent Events for typical homelab and self-hosted applications.
- Basic but usable observability: logs that clearly show authentication and authorization decisions, request outcomes, and minimal health checks and status indicators in the UI.

### Growth Features (Post-MVP)

- More polished observability, including built-in dashboards, richer metrics exports, and easier log analysis for troubleshooting.
- Simplified horizontal scaling patterns, such as templates or guidance for multi-instance setups and configuration synchronization across instances.
- Additional identity provider integrations and quality-of-life UI improvements for managing many services and roles.

### Vision (Future)

- A batteries-included, zero-trust-style front door for self-hosted and small-team environments, with opinionated best-practice defaults, deep observability, and smooth multi-instance scaling, while remaining approachable for a single homelab operator.

## User Journeys

**Homelab Admin – First-Time Setup and First App**

A technically comfortable homelab user spins up a new VM or bare-metal box and wants a front door that gives them enterprise-grade SSO and authorization without a weekend of YAML. They clone the repository, skim the README, and run a single `docker compose up` to deploy the stack. No extra scripting, no manual configuration files. The only external preparation is pointing a wildcard DNS record (`*.domain.com`) at the server’s IP address.

Once the containers are up, they open a browser and go to `https://admin.domain.com`. Because this is the very first visit, they are guided through an initial setup flow. They create a local admin account (email, strong password, with optional two-factor authentication later) and are dropped into a simple wizard that focuses on wiring identity. The wizard prompts them to choose an identity provider (Azure Entra, Google, or an existing Keycloak), then walks them through entering the minimal required fields (client ID, client secret, issuer, callback URL), with copy-paste-ready redirect URIs and inline hints. If they do not yet have an identity provider, the UI points out that a bundled Keycloak instance is available from the same Docker Compose stack (with clear “optional” labeling) and links to a short setup guide.

With identity configured, the admin lands on an empty but clear Applications view. They click Add application and go through a structured flow: name the application, provide the upstream URL, pick the external hostname (for example, `app1.domain.com` using the wildcard), select the authentication mode (such as login required), and either pick existing roles or define a small set of roles and basic rules (for example, admins only or family plus admins). Before saving, they can run a connection test that checks upstream reachability, TLS, and identity provider integration, with clear, actionable error messages if something is wrong (for example, wrong upstream URL, invalid certificate, or misconfigured client and secret).

Once the application is created, the admin can immediately browse to `https://app1.domain.com`, go through the SSO flow, and confirm the app loads correctly. Back in the UI, a lightweight traffic and access view starts to show requests in near real time: they can see who has accessed the app recently, whether any logins failed, and basic health indicators (route up or down, error rates). Without touching Docker or configuration files again, they now feel confident that new applications can be added in the same way and that misconfigurations will be surfaced clearly rather than silently breaking access.

### Journey Requirements Summary

- Single-command deployment path (Docker Compose) with no required manual configuration editing.
- Documentation and UX that assume a wildcard DNS record (`*.domain.com`) pointing to the box, with this requirement clearly explained.
- First-time visit to `https://admin.domain.com` triggers an initial setup wizard for creating a local admin account and wiring an identity provider (Azure Entra, Google, or existing Keycloak).
- Optional bundled Keycloak instance in the Docker Compose stack, clearly labeled and documented as an option rather than a requirement.
- Clear, guided identity provider configuration flow with copy-paste-ready redirect URIs and validation of client ID, client secret, and issuer values.
- Add application wizard that covers application name, upstream URL, external hostname, authentication mode, role definition or selection, and a connection test step that validates connectivity and authentication.
- Robust error handling for common misconfigurations (incorrect upstream URL, TLS handshake issues, invalid identity provider credentials, redirect mismatch) with concrete, UI-visible guidance.
- Built-in basic monitoring with a recent-access list, failed-login counts, and simple per-application health and traffic indicators.

## Innovation & Novel Patterns

### Detected Innovation Areas

- Policy-as-UI for identity-aware reverse proxying: instead of configuring authentication and authorization via YAML, sidecars, or complex configuration files, skaild2 treats authentication and routing policy as a first-class, UI-driven model that can be safely changed at runtime without restarts.
- Enterprise-grade zero-trust front door for homelab: the product applies Pomerium-class concepts such as SSO, role-based access control, and identity-aware routing to homelab and small self-hosted environments with a one-command deployment path and simple wildcard DNS, assuming very limited infrastructure time per user.
- Deep RBAC and observability in a single box: role modeling, request-level auditability, and per-application health and traffic views are integrated into the same admin surface as routing and identity provider configuration, rather than spread across multiple tools such as reverse proxy, identity provider, and logging stack.

### Market Context & Competitive Landscape

- Existing tools such as Nginx and Traefik with authentication add-ons, Authelia, Pomerium, and Keycloak-fronted setups generally assume either high comfort with text-based configuration and orchestration or a heavier enterprise platform context with dedicated operations time.
- Homelab and small self-hosted users typically stitch together reverse proxies, identity providers, and logging and metrics manually, resulting in fragile setups where authentication policy is encoded in multiple places, observability requires separate stacks, and configuration changes are risky because they are opaque and tied to restarts.
- skaild2 aims to define a new pattern for this segment: a zero-trust-style gateway in a box, with one deployment, one admin UI, and a clean mental model for routes, authentication, authorization, and visibility.

### Validation Approach

- Time to first secure application: measure how long a technically comfortable homelab admin with no prior Pomerium or Authelia experience takes to go from `docker compose up` and wildcard DNS to a correctly protected application with SSO and a basic role model.
- Configuration safety and confidence: observe whether users are willing to make changes such as adding routes and adjusting roles directly in the UI without resorting to manual backups or long maintenance windows, and track misconfiguration-related outages.
- Recommendation behavior: track whether power users organically recommend skaild2 as the default front door in homelab and self-hosting communities, specifically calling out the new pattern of UI-driven policy and integrated observability rather than just another reverse proxy.

### Risk Mitigation

- Fallback to known-good patterns: where the new pattern is too unfamiliar or risky, allow users to fall back to simpler authentication modes or more explicit, expert-friendly views, such as an advanced policy view or exportable configuration, without abandoning the UI.
- Incremental rollout of advanced features: start with a minimal but solid expression of policy-as-UI and observability, adding more advanced constructs only after usability testing, to avoid a complex, enterprise-only feel.
- Guardrails against misconfiguration: build validation, simulations, and clear error surfacing into the UI so that new policy edits are checked before they can break access, and any issues are diagnosable without deep protocol knowledge.

## Identity-Aware Reverse Proxy Specific Requirements

### Project-Type Overview

- Identity-aware reverse proxy with an admin web UI, providing SSO, role-based access control, and policy-as-UI for homelab and small-team environments.
- Designed as a zero-trust-style gateway in a box with simple deployment and strong security and observability.

### Technical Architecture Considerations

- Control plane
  - Admin application (frontend and backend) runs as its own service.
  - Dedicated relational database for configuration, policies, and metadata.
  - API surface primarily for the UI in v1, with no public SDK exposed.

- Data plane (reverse proxy layer)
  - One or many stateless reverse proxy instances, horizontally scalable behind a single entrypoint (for example, via DNS or a load balancer).
  - Proxies consume configuration and policies from the control plane and shared state store; no per-node manual configuration.
  - Designed to handle HTTP and HTTPS, WebSockets, Server-Sent Events, and typical homelab application patterns.

- Identity providers
  - External identity providers supported in v1 include Azure Entra, Google, and existing Keycloak deployments.
  - Optional bundled Keycloak (plus its database) as part of the Docker Compose stack, clearly documented as optional rather than required.

- Shared state and sessions
  - Centralized session store (Redis or equivalent) so multiple proxy instances can share login state and support horizontal scaling without sticky sessions.

- Observability and logs
  - Logs for admin actions, authentication decisions, and request and response-level events collected in a consistent format.
  - Minimal built-in way to view and filter logs and basic metrics from the admin UI, with optional integration paths to external log stacks in later versions.

### Deployment Model (v1 vs v2)

- v1 (primary focus)
  - Officially supported deployment is a single-host Docker Compose stack that includes the admin app, its database, one or more proxy containers, Redis or an equivalent store, and optional Keycloak with its database.
  - Kubernetes and multi-host clustering are explicitly out of scope for v1 beyond the general property that components run in containers.

- v2 (future direction)
  - Multi-host deployments managed via Kubernetes and Helm, with clear separation between control plane and data plane and first-class support for running multiple proxy nodes across a cluster.

### Implementation Considerations

- No user extensions in v1
  - v1 is configuration-only: no plugin system, custom code hooks, or arbitrary webhooks are required or supported for core features.
  - Policy expression is handled via the built-in model and UI; advanced extension mechanisms are deferred to later versions.

- Safety and upgrade path
  - Versioned configuration schema to allow safe upgrades between releases.
  - Clear migration story from single-host Docker Compose to future Kubernetes-based multi-host setups in v2 and beyond, without blocking the v1 experience.

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

- MVP approach: experience plus pattern validation — deliver a full front door in a box experience for a single homelab user and validate that policy-as-UI with integrated observability is actually better than the current do-it-yourself stack.
- Resource assumption: effectively a solo full-stack developer with infrastructure skills, so v1 scope must stay tight and composable.

### MVP Feature Set (Phase 1)

- Core user journeys supported:
  - First deployment on a single host via Docker Compose plus wildcard DNS setup.
  - First-time admin and identity provider wiring via a wizard at `https://admin.domain.com`.
  - Creating and protecting at least one application, including route definition, authentication mode, roles, and test flow.
  - Basic monitoring of access and health for that application.

- Must-have capabilities:
  - Admin UI, API, and database for configuration and policies.
  - One or more stateless reverse proxy instances sharing sessions via Redis or an equivalent store.
  - Support for HTTP and HTTPS, WebSockets, and Server-Sent Events to typical homelab applications.
  - Single sign-on via Azure Entra, Google, and existing Keycloak, with optional bundled Keycloak included in the Docker Compose stack.
  - Basic observability with logs and a simple per-application access and health view in the admin UI.
  - Safe, UI-driven policy changes without requiring service restarts.

### Post-MVP Features

- Phase 2 (Growth):
  - Richer observability with dashboards, metrics export, and improved log analysis.
  - More polished multi-instance patterns on a single host, including load balancing and resilience.
  - Additional identity providers and user experience improvements for managing many applications and roles.

- Phase 3 (Expansion):
  - First-class Kubernetes and Helm-based multi-host support with clear separation between control plane and data plane.
  - Extensibility hooks such as webhooks, advanced policy expressions, and potential plugin surfaces.
  - Deeper integrations with external logging and monitoring stacks.

### Risk Mitigation Strategy

- Technical risks: complexity of authentication flows, session sharing, and multi-protocol support are addressed by keeping v1 scope tight, prioritizing correctness and observability, and deferring exotic edge cases to later phases.
- Market risks: homelab users might initially see skaild2 as just another reverse proxy, so the product leans heavily on the new pattern of UI-driven policy and observability and measures recommendation behavior early.
- Resource risks: with a solo developer capacity, aggressively protect MVP boundaries from scope creep and push non-essential nice-to-have features into Phase 2 and Phase 3.

## Functional Requirements

### Access and Identity

- FR1: Admin users can configure one or more external identity providers, such as Azure Entra, Google, or existing Keycloak, for use across all protected applications.
- FR2: Admin users can test an identity provider configuration, for example with a test login, before enabling it for production traffic.
- FR3: Admin users can define roles and groups that represent access levels used across applications.
- FR4: Admin users can map identity provider attributes, such as claims and groups, to internal roles.
- FR5: End users can authenticate to protected applications via single sign-on using a configured identity provider.
- FR6: End users can be denied or granted access to a specific application based on their assigned roles.

### Application and Routing Management

- FR7: Admin users can register a new application by specifying its upstream address and desired external hostname.
- FR8: Admin users can define one or more routes per application, including path-based routing when needed.
- FR9: Admin users can choose the access mode for a route, such as public, login required, or role-restricted.
- FR10: Admin users can run a connection test for an application or route that validates upstream reachability, TLS behavior, and basic authentication integration.
- FR11: Admin users can enable or disable an application or route without deleting its configuration.
- FR12: The system can route HTTP and HTTPS, WebSocket, and Server-Sent Events traffic for configured applications according to defined routes.

### Configuration and Policy Management

- FR13: Admin users can view and edit access policies for each application and route through the UI without editing configuration files.
- FR14: Admin users can preview the effective policy for a given application or route, including which roles have access.
- FR15: Admin users can review a history of configuration and policy changes with timestamps and actor information.
- FR16: The system can apply configuration and policy changes without requiring a restart of the reverse proxy instances.
- FR17: The system can validate configuration changes before applying them and surface validation errors to the admin user.

### Monitoring and Observability

- FR18: Admin users can view recent access activity per application, including which users accessed which routes and when.
- FR19: Admin users can view counts of failed authentication or authorization attempts per application or route.
- FR20: Admin users can view basic health status for each application or route, for example up, down, or degraded, based on connectivity and error rates.
- FR21: Admin users can filter or search log and event views by application, route, user, or time range.
- FR22: The system can export or expose logs in a format suitable for integration with external logging or monitoring tools.

### Deployment and Operations

- FR23: Admin users can deploy the full stack on a single host using a documented Docker Compose configuration.
- FR24: Admin users can configure the system to operate behind a wildcard DNS entry that fronts one or more applications.
- FR25: The system can operate with multiple reverse proxy instances on the same host sharing session state via a supported session store.
- FR26: Admin users can perform configuration backup and restore operations for the control plane, such as exporting and importing configuration and policies.
- FR27: The system can perform safe configuration migrations when upgrading between compatible versions.

### Administration and Security

- FR28: Admin users can create, update, and deactivate administrator accounts for access to the admin UI.
- FR29: Admin users can configure authentication requirements for admin access, such as password policy and optional second factor when available.
- FR30: Admin users can view an audit trail of sensitive admin actions, such as policy changes, identity provider changes, and application enable or disable actions.
- FR31: Admin users can restrict which administrators are allowed to perform specific high-risk actions, such as managing identity providers or deleting applications.
- FR32: The system can enforce secure defaults for new configurations, such as requiring authentication for new applications unless explicitly configured otherwise.

## Non-Functional Requirements

### Performance

- NFR1: When the reverse proxy and upstream application run on the same physical host, the additional median latency introduced by skaild2 per HTTP(S) request is under 10 ms under normal homelab workloads.
- NFR2: Under sustained normal load with multiple concurrent users and long-lived WebSocket and Server-Sent Events connections, skaild2 maintains the latency target from NFR1 without becoming the primary performance bottleneck on typical homelab hardware.

### Security

- NFR3: All external access to the admin interface and protected applications is served over TLS; plaintext HTTP is either redirected or disabled.
- NFR4: Administrator credentials and identity provider secrets are stored using industry-standard protections, such as hashed passwords and encrypted secrets at rest.
- NFR5: All authentication and authorization decisions, as well as sensitive admin actions such as policy changes, identity provider changes, and application enable or disable actions, are auditable via logs that include actor, action, target, and timestamp.
- NFR6: Default configuration favors secure behavior so that new applications require authentication by default unless explicitly configured otherwise.

### Scalability

- NFR7: The default Docker Compose deployment supports running at least two reverse proxy instances concurrently on a single host, sharing sessions via the configured session store.
- NFR8: Configuration and policy changes propagate correctly to all running reverse proxy instances without requiring instance restarts and without breaking active user sessions.
- NFR9: Under typical homelab traffic with at least two reverse proxy instances, skaild2 continues to meet the performance targets defined in the performance non-functional requirements.

### Integration

- NFR10: skaild2 exposes Prometheus-compatible metrics, for example via an HTTP `/metrics` endpoint, including at minimum request counts, error counts, authentication and authorization failures, and basic per-application traffic indicators.
- NFR11: Log formats and metric labels are structured in a way that allows straightforward ingestion into common observability stacks, such as Prometheus and Grafana or Loki and ELK, without custom parsing.
