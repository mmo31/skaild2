# skaild2

A reverse proxy gateway with identity-aware access control, built for homelabs and self-hosters.

## Prerequisites

- Rust toolchain 1.75 or later (2021 edition)
- Node.js 18 LTS or later + npm
- Docker 24.0+ and Docker Compose v2.20+

## Quick Start

### For Development

1. Clone the repository and set up environment:
```bash
git clone <repo-url>
cd skaild2
cp .env.example .env  # If .env.example exists, otherwise defaults are used
```

2. Start the full stack with Docker Compose:
```bash
docker compose -f docker-compose.dev.yml up --build
```

This will start:
- **Postgres** (port 5432) - Configuration and policy database
- **Redis** (internal) - Session storage
- **Control Plane** (internal) - Admin API on port 8080
- **Gateway** (ports 80/443) - Reverse proxy (future stories)
- **Admin UI** (port 5173) - Web interface for management

3. Open the admin UI in your browser:
```
http://localhost:5173
```

4. **First-Time Setup**: Create your admin account
   - On first launch, you'll see the setup page
   - Enter your email and create a secure password
   - Password requirements:
     - Minimum 12 characters
     - At least one uppercase letter
     - At least one lowercase letter
     - At least one number
     - At least one special character
   - Click "Create Admin Account"
   - You'll be redirected to the login page

5. **Sign In**: Use your newly created credentials to access the dashboard

### Environment Variables

The following environment variables can be configured (defaults shown):

**Database:**
- `DATABASE_URL` - PostgreSQL connection string (default: `postgres://skaild2:skaild2_dev_password@postgres:5432/skaild2`)

**Redis:**
- `REDIS_URL` - Redis connection string (default: `redis://redis:6379`)

**Admin UI:**
- `VITE_API_URL` - Control plane API URL (default: `http://localhost:8080`)

**Control Plane:**
- `FRONTEND_URL` - Admin UI origin allowed by CORS (default: `http://localhost:5173`)
- `COOKIE_SECURE` - Set `true`/`1` to mark session cookies `Secure` when running behind HTTPS (default: `false`)

**Logging:**
- `RUST_LOG` - Rust log level (default: `debug` in dev, `info` in prod)

## Project Structure

## Project Structure

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

## Authentication & Security

### Admin Account Management

- **First Admin**: Created through the setup wizard on first launch
- **Password Requirements**: Strong passwords enforced (12+ chars, mixed case, numbers, special chars)
- **Password Hashing**: Argon2 (more secure than bcrypt)
- **Session Management**: Redis-backed sessions with 1-hour inactivity timeout
- **Session Cookies**: httpOnly, secure (when HTTPS), SameSite=Lax

### API Endpoints

**Setup:**
- `GET /api/setup/status` - Check if setup is complete
- `POST /api/setup/create-admin` - Create first admin account

**Authentication:**
- `POST /api/auth/login` - Login with email/password
- `POST /api/auth/logout` - Destroy session
- `GET /api/auth/me` - Get current authenticated admin

**Applications (auth required):**
- `POST /api/applications` - Register a new upstream application
- `GET /api/applications` - List all registered applications
- `GET /api/applications/:id` - Get a single application by ID
- `PUT /api/applications/:id` - Update name, upstream URL, or hostname

> **Hostname uniqueness**: each application must have a distinct `hostname` value (enforced by a unique DB constraint). The hostname is the external-facing domain under your wildcard certificate that the gateway will route to this upstream.

## Docker Compose Deployment

## Docker Compose Deployment

### Available Compose Files

- `docker-compose.dev.yml` - Development mode with live reload and exposed ports
- `docker-compose.prod.yml` - Production build (if exists)
- `docker-compose.yml` - Base configuration

### Development Topology

The development stack (`docker-compose.dev.yml`) includes:

- `gateway` – edge HTTP/HTTPS entrypoint (only service exposed to the host on ports 80/443)
- `control-plane` – internal admin/control APIs (port 8080, called by admin-ui)
- `postgres` – configuration and policy store (port 5432 exposed for dev access)
- `redis` – session storage (internal only)
- `admin-ui` – React development server (port 5173)

All services run on the private `skaild2-net` bridge network. The control-plane and gateway services share access to Postgres and Redis.

### Database Migrations

Migrations run automatically when the control-plane starts:
- Located in `migrations/` directory
- Uses sqlx migrate system
- First migration creates the `admins` table

### Stack restart preserves data

When you restart the stack, your admin account and all configuration are preserved in the Postgres volume:

```bash
docker compose -f docker-compose.dev.yml down
docker compose -f docker-compose.dev.yml up
# Your admin account still works!
```

To completely reset (⚠️ deletes all data):
```bash
docker compose -f docker-compose.dev.yml down -v
```

## Troubleshooting

### Admin UI can't connect to API

**Symptom:** Login fails with network error, or setup page doesn't load.

**Cause:** Control plane not accessible or CORS issue.

**Solution:**
1. Check control-plane is running: `docker compose -f docker-compose.dev.yml ps`
2. Check control-plane logs: `docker compose -f docker-compose.dev.yml logs control-plane`
3. Verify VITE_API_URL in docker-compose.dev.yml points to `http://localhost:8080`
4. Try accessing API directly: `curl http://localhost:8080/health`

### Database connection failed

**Symptom:** Control plane fails to start with database connection error.

**Cause:** Postgres not ready or wrong credentials.

**Solution:**
1. Check Postgres is running: `docker compose -f docker-compose.dev.yml ps postgres`
2. Check Postgres logs: `docker compose -f docker-compose.dev.yml logs postgres`
3. Verify DATABASE_URL matches Postgres credentials in docker-compose.dev.yml
4. Ensure Postgres health check passes before control-plane starts (depends_on with condition)

### Setup already complete but can't login

**Symptom:** Setup says "already complete" but you don't remember the password.

**Cause:** Admin account exists in database.

**Solution:**
1. Stop the stack
2. Delete the Postgres volume: `docker compose -f docker-compose.dev.yml down -v`
3. Start fresh: `docker compose -f docker-compose.dev.yml up --build`

### Redis connection failed

**Symptom:**  Control plane logs show Redis connection errors.

**Cause:** Redis not running or wrong URL.

**Solution:**
1. Check Redis is running: `docker compose -f docker-compose.dev.yml ps redis`
2. Check health: `docker compose -f docker-compose.dev.yml exec redis redis-cli ping` (should return "PONG")
3. Verify REDIS_URL in docker-compose.dev.yml

### Port already in use

**Symptom:** Docker compose fails with "port is already allocated"

**Cause:** Another service is using the same port.

**Solution:**
1. Check what's using the port: `sudo lsof -i :5173` (or :8080, :5432, etc.)
2. Stop the conflicting service
3. OR change the port in docker-compose.dev.yml

## Development Workflow

### Testing

Run Rust tests:
```bash
cargo test --workspace
```

Run integration tests (requires test database):
```bash
DATABASE_URL=postgres://skaild2:skaild2_dev_password@localhost:5432/skaild2_test cargo test -p control-plane --test auth_flow
```

Run frontend tests:
```bash
cd admin-ui
npm test
```

### Building

Build Rust services:
```bash
cargo build --release
```

Build admin UI for production:
```bash
cd admin-ui
npm run build
```

## What's Next?

Once you have the stack running and can log in to the admin UI, you're ready for:

1. **Story 2.1**: Register your first upstream application
2. **Story 2.2**: Define routes and access modes
3. **Story 3.1**: Connect an identity provider (OAuth/OIDC/SAML)
4. **Story 3.2**: Set up role-based access control

## License

[Add your license here]
