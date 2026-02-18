# Story 1.0: Initial Project Setup from Starter Templates

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer working on skaild2,
I want the project scaffolded using the agreed Rust workspace and React/Vite/Tailwind/Mermaidcore stack with a base Docker Compose file,
So that all subsequent deployment and feature work builds on a consistent, reproducible foundation.

## Acceptance Criteria

1. Given I have a development machine with a supported OS, Rust toolchain, Node.js, Docker, and Docker Compose installed, when I follow the documented setup steps to clone the repository and initialize the project, then the repo contains a Rust Cargo workspace with at least `gateway`, `control-plane`, and `shared` crates configured and buildable.
2. Given the workspace is initialized, when I open the admin UI project, then there is a React + TypeScript app created with Vite, wired to use TailwindCSS and the Mermaidcore design tokens from the UX specification.
3. Given the backend services and admin UI projects exist, when I open the provided `docker-compose.yml`, then it defines services for the gateway, control-plane, and Postgres (and any required supporting services) on a private network, suitable as the baseline for later deployment stories.
4. Given the initial setup has completed successfully, when I run the documented commands to build the Rust services and start the dev stack (locally or via Compose), then the code builds without errors and the containers start to the point where subsequent deployment and configuration stories can proceed.

## Tasks / Subtasks

- [ ] Initialize Rust workspace and crates
  - [ ] Create top-level `Cargo.toml` configured as a workspace
  - [ ] Add `gateway` crate for the reverse proxy / data plane
  - [ ] Add `control-plane` crate for admin API and background workers
  - [ ] Add `shared` crate for domain models, configuration types, and database layer
  - [ ] Verify `cargo build` at workspace root builds all crates successfully
- [ ] Scaffold React + Vite + TypeScript admin UI
  - [ ] Create admin UI project using Vite + React + TypeScript
  - [ ] Integrate TailwindCSS with configuration matching Mermaidcore tokens from the UX spec
  - [ ] Wire a basic shell for the admin dashboard (navigation, placeholder screens for Dashboard, Routes, Identity, Certificates, Settings)
- [ ] Wire Docker Compose baseline
  - [ ] Ensure `docker-compose.yml` defines services for gateway, control-plane, and Postgres on a private network
  - [ ] Add volumes for Postgres data and any required configuration
  - [ ] Configure environment variables and ports so only the gateway is directly exposed to the host for HTTP/HTTPS
- [ ] Document developer workflows
  - [ ] Add `README` or `CONTRIBUTING` section that documents required toolchain versions (Rust, Node, Docker, Docker Compose)
  - [ ] Document commands to build the Rust workspace, run the admin UI dev server, and bring up the Compose stack for local development
  - [ ] Describe how the control-plane and gateway connect to Postgres in dev

## Dev Notes

The goal of this story is to establish a clean, composable foundation that lines up with the architecture and PRD:

- Rust workspace with clear separation of concerns between `gateway` (data plane) and `control-plane` (control plane), sharing types and persistence code through a `shared` crate.
- React + TypeScript admin UI using Vite and TailwindCSS, styled according to the Mermaidcore UX design tokens to keep visual language consistent from the beginning.
- Docker Compose as the primary deployment and local dev topology, with gateway as the only exposed edge service, and Postgres as the configuration and policy store.

### Project Structure Notes

The following structure is recommended for this story (adapt as needed in later stories, but keep the roles clear):

- Workspace root
  - `Cargo.toml` (workspace definition)
  - `crates/`
    - `gateway/`
    - `control-plane/`
    - `shared/`
  - `admin-ui/` (React + Vite + TailwindCSS project)
  - `docker-compose.yml`

Key expectations from the architecture document:

- Gateway is the only public HTTP(S) entrypoint; it will eventually own TLS termination, routing, and per-request auth.
- Control-plane exposes internal APIs and background workers for DNS/ACME, IdP synchronization, and configuration management.
- Postgres is the canonical store for routes, IdPs, roles, policies, and cert metadata and is not exposed directly to the public network.
- Deployment for v1 is Docker Compose on a single host; Kubernetes is a later phase but should remain feasible given the service boundaries you establish here.

### References

- Product requirements and user journeys: see `_bmad-output/planning-artifacts/prd.md` (deployment model, first-time setup journey, and MVP scope).
- Epics and acceptance criteria: see `_bmad-output/planning-artifacts/epics.md` (Epic 1 and Story 1.0).
- Architecture decisions: see `_bmad-output/planning-artifacts/architecture.md` (stack selection, workspace split, gateway/control-plane roles, and Docker Compose-first topology).
- UX design system and Mermaidcore tokens: see `_bmad-output/planning-artifacts/ux-design-specification.md` (Tailwind configuration, CSS variables, and component guidelines).

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

- N/A for initial story; use this section in future stories to record any issues found during implementation.

### Completion Notes List

- Initialize once the workspace, admin UI, and Compose baseline are in place and buildable.

### File List

- `Cargo.toml` (workspace root)
- `crates/gateway/Cargo.toml`
- `crates/control-plane/Cargo.toml`
- `crates/shared/Cargo.toml`
- `admin-ui` project scaffold (Vite + React + TypeScript + TailwindCSS)
- `docker-compose.yml` updated or confirmed to include gateway, control-plane, and Postgres services
