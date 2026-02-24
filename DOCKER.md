# Docker Compose Setup

This project has two Docker Compose configurations for different environments.

## Development Setup

**File**: `docker-compose.dev.yml`

Features:
- Admin UI with hot-reload (mounted source code)
- Debug logging enabled
- PostgreSQL exposed on port 5432 for direct access
- Admin UI accessible at http://localhost:5173

```bash
docker compose -f docker-compose.dev.yml up
```

## Production Setup

**File**: `docker-compose.prod.yml`

Features:
- Admin UI built and served with nginx
- Info-level logging
- Auto-restart enabled
- Admin UI accessible at http://localhost:3000

```bash
docker compose -f docker-compose.prod.yml up -d
```

## Services

### Gateway
- Ports: 80 (HTTP), 443 (HTTPS)
- Handles incoming requests and routes to backend services

### Control Plane
- Internal service (not exposed publicly)
- Port 8080 (internal only)
- Accessible only via gateway or within Docker network

### Admin UI
- **Dev**: Port 5173 with live reload
- **Prod**: Port 3000 with optimized build

### PostgreSQL
- **Dev**: Port 5432 exposed for development tools
- **Prod**: Internal only
- Database: skaild2
- User: skaild2
- Password: skaild2

## Environment Variables

Create a `.env` file in the project root for additional configuration.

## Notes

- The base `docker-compose.yml` is kept for backward compatibility but it's recommended to use the dev or prod versions.
- Control plane is intentionally not exposed publicly - all access should go through the gateway.
