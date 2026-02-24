# skaild2

Initial project scaffold for the skaild2 gateway and control-plane.

## Prerequisites

- Rust toolchain 1.75 or later (2021 edition)
- Node.js 18 LTS or later + npm
- Docker 24.0+ and Docker Compose v2.20+

## Rust workspace

This repo uses a Cargo workspace with three crates:

- `crates/gateway` – reverse proxy / data plane (binary)
- `crates/control-plane` – admin API and background workers (binary)
- `crates/shared` – shared domain models and configuration/database types (library)

From the repo root:

```bash
# Build shared types and services
cargo build -p shared -p gateway -p control-plane

# Run shared crate tests
cargo test -p shared
```

## Admin UI (React + Vite + Tailwind)

The admin UI lives in `admin-ui/` and is built with React, TypeScript, Vite, and TailwindCSS configured with Mermaidcore design tokens.

```bash
cd admin-ui
npm install
npm run dev   # start dev server
npm run build # production build
npm test      # run Vitest + Testing Library tests
```

## Docker Compose baseline

The root `docker-compose.yml` defines the baseline local topology:

- `gateway` – edge HTTP/HTTPS entrypoint (only service exposed to the host)
- `control-plane` – internal admin/control APIs
- `postgres` – configuration and policy store

All services run on the private `skaild2-net` bridge network.

Both `gateway` and `control-plane` use the `SKAILD2_DB_URL` environment variable to connect to the `postgres` service
(`postgres://skaild2:skaild2@postgres:5432/skaild2`), so they can share the same configuration and policy store in development.

### Starting the stack

1. Copy the environment template:
```bash
cp .env.example .env
```

2. Build and start the services:
```bash
docker compose up -d --build
```

3. Check service health:
```bash
curl http://localhost/health  # Gateway health check
```

4. View logs:
```bash
docker compose logs -f
```

5. Stop the stack:
```bash
docker compose down
```

Once running, later stories will wire routing, identity providers, and configuration flows on top of this baseline.
