# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Full-stack IT infrastructure admin tool with a Rust backend (Axum) and Vue 3 frontend. Tracks applications, services, infrastructure, domains, people, network shares, technology stacks, and healthchecks with Uptime Kuma integration. The frontend is compiled and embedded into the Rust binary for single-binary deployment.

## Build Commands

### Using `just` (preferred)

```bash
just dev              # Start both backend + frontend dev server
just watch            # Backend with cargo-watch auto-reload
just watch-frontend   # Frontend build in watch mode (bun run watch)
just create-db        # Create SQLite database and run migrations
just migrate          # Run pending migrations
just reset-db         # Reset database completely
just prepare          # Generate .sqlx query cache for offline compilation
just docker-build     # Build Docker image
just docker-run       # Run Docker container
```

### Manual commands

```bash
cargo build                    # Build backend
cargo run                      # Run server (default: localhost:8080)
cd frontend && bun install     # Install frontend dependencies
cd frontend && bun run dev     # Vite dev server (proxies /api to localhost:8080)
cd frontend && bun run build   # Build to frontend-dist/ (embedded in backend)
cd frontend && bun run format  # Format with Prettier
cd frontend && bun run check   # TypeScript type check (vue-tsc)
```

### Development Workflow

1. `just dev` starts both backend and frontend with Ctrl+C cleanup
2. Vite dev server proxies `/api` requests to the Rust backend on port 8080
3. Before deployment: `cd frontend && bun run build` then rebuild the Rust binary
4. Swagger UI available at `/api/docs` when backend is running

## Architecture

### Backend Layer Pattern

Every entity follows the same three-layer pattern:

1. **`api/{entity}.rs`** - Axum handlers with utoipa OpenAPI annotations. Uses `Query`, `Path`, `State` extractors. Relationship endpoints follow `/{entity}/{id}/{relation}` pattern.
2. **`service/{entity}.rs`** - Business logic with async functions: `list()`, `get()`, `create()`, `update()`, `delete()`, plus relationship CRUD. All use parameterized sqlx queries.
3. **`models/{entity}.rs`** - Four structs per entity: `Entity` (`#[derive(FromRow, Serialize, ToSchema)]`), `CreateEntity`, `UpdateEntity` (all fields optional), `EntityWithRelations` (entity + Vec of related items). Also `EntityRelation` lightweight structs with id, name, and notes.

Shared models: `PaginationParams` (page/per_page/search) and `PaginatedResponse<T>` in `models/mod.rs`.

### AppState

```
AppState {
    pool: SqlitePool,
    config: Config,
    uptime_state: Arc<RwLock<HashMap<...>>>  // In-memory Kuma heartbeats
    uptime_tx: broadcast::Sender             // SSE fan-out for uptime events
    kuma_refresh_tx: watch::Sender<()>       // Signal Kuma poller to reconnect
}
```

### Kuma Integration (`src/kuma.rs`)

Custom Socket.IO client for Uptime Kuma (replaces kuma-client crate due to float deserialization issues). Polls heartbeat data, broadcasts via channel, maintains ~1 hour of heartbeats per monitor in memory. Healthchecks have a `kuma_dirty` flag tracking sync state.

### Frontend Patterns

- **API client** (`api/index.ts`): Namespaced objects (`applicationsApi`, `servicesApi`, etc.) with typed methods. Central `request<T>()` wrapper around fetch.
- **Components**: Reusable `EntityList`, `EntityDetail`, `EntitySelector` in `components/common/`. Separate form components in `components/forms/`. Views in `views/`.
- **Composables**: `useUptime()` manages SSE connection to `/api/healthchecks/uptime/stream` with singleton pattern and auto-cleanup.
- **State**: No global store (no Pinia/Vuex) — components call API directly and use local `ref`/`reactive`.
- **Styling**: Tailwind CSS v4 + DaisyUI 5 component classes. Use DaisyUI semantic classes (e.g., `btn btn-primary`, `card`, `badge`) over raw Tailwind where possible.
- **3D**: Three.js via TresJS for mascot/model viewer.

### Database

- SQLite with sqlx. Migrations in `/migrations/` (auto-run on startup).
- Junction tables for many-to-many relationships (e.g., `application_infra`, `application_service`) with optional `notes` field.
- Healthcheck has XOR constraint: must reference either `application_id` or `service_id`, not both.
- Unique constraints on entity name + environment where applicable.

### Error Handling

`src/error.rs` defines an `Error` enum with `thiserror`. Maps to HTTP status codes via `IntoResponse`: NotFound→404, ValidationError→400, Conflict→409, InternalError→500. SQLx unique constraint violations automatically become 409 Conflict.

## Configuration

Config is loaded by `Config::load()` (`src/config.rs`) via `figment`, layering:
**built-in defaults < `auto.toml` < environment** (env wins). `.env` (falling back
to `dev.env`) is loaded into the process environment first.

Environment variables / core keys:

- `HOST` / `PORT` - Server bind address (default: `0.0.0.0:8080`)
- `DATABASE_URL` - SQLite connection (default: `sqlite://data/data.db`)
- `KUMA_URL` - Uptime Kuma instance URL
- `KUMA_USERNAME` / `KUMA_PASSWORD` - Kuma authentication credentials

## Authentication

In-app auth lives in `src/auth/` (types, argon2 password hashing, server-side
sessions + `HttpOnly` cookie, the `AuthUser` extractor, and session/role
middleware). It replaces the old reverse-proxy Basic Auth. Both password and
OIDC logins normalise into an `AuthUser` and a session row; subsequent requests
carry only the opaque session token. See `AUTHENTICATION.md` for the design.

- **Opt-in.** Auth is enforced only when a login method is enabled
  (`auth_password_enabled` or `auth_oidc_enabled`); both default to false. With
  neither enabled, `api_routes` mounts the data routes unauthenticated and skips
  the session/admin route groups, so the app serves open (proxy-only) as before.
  The frontend mirrors this via `authEnabled` in `useAuth.ts` (no route guards,
  everyone `canEdit`).

- **Roles** (`user_roles`, denormalised onto `sessions`): `admin` (all),
  `editor` (all except `/api/admin/*` user management), `viewer` (read-only).
- **Route tiers** (wired in `src/api/mod.rs`): public (login, logout,
  set-password, reset-request, oidc/*, link, config, version, health);
  session-only any-role (`/auth/me`, `/auth/change-password`); data routes
  (GET any role, mutations editor+ via `require_role`); admin (`require_admin`).
- **Accounts** are admin-created only (no self-registration). Users set their own
  password via a one-time setup link (`password_setup_tokens`); forgotten-password
  requests (`password_reset_requests`) surface to admins who issue a new link.
- **First admin** is seeded once on startup from `bootstrap_admin_*` when the
  `users` table is empty.
- **OIDC** auto-creates a `viewer` on first login, or routes to account-linking
  when the email matches an existing password account.

Auth config keys (env names; same keys work in `auto.toml`):

- `SESSION_HOURS` - session lifetime in hours (default 168)
- `AUTH_PASSWORD_ENABLED` / `AUTH_OIDC_ENABLED` - which login methods are offered
- `BOOTSTRAP_ADMIN_USERNAME` / `BOOTSTRAP_ADMIN_PASSWORD` - first-admin seed
- `OIDC_ISSUER_URL` / `OIDC_CLIENT_ID` / `OIDC_CLIENT_SECRET` - provider + confidential client
- `OIDC_REDIRECT_URL` - optional; defaults to `base_url` + `auth::oidc::OIDC_CALLBACK_PATH`
  (`/api/auth/oidc/callback`), so it rarely needs setting

The frontend mirrors auth state in `composables/useAuth.ts` (singleton, like
`useConfig.ts`); `router.ts` guards routes; `request()` in `api/index.ts` sends
the cookie and bounces 401s to `/login`.

### Configurable defaults & options (`auto.toml`)

`Config.defaults` and `Config.options` (see `src/config.rs`) make the app
portable across organizations. `[defaults.*]` are the **single source of truth**
for entity-creation defaults: backend create handlers fill omitted fields from
them (so `POST {name}` works), and the same values are served at `GET /api/config`.
`[options.*]` are dropdown value→label maps (order-preserving `IndexMap`).

The frontend fetches `/api/config` once at boot (`composables/useConfig.ts`,
gated in `App.vue`) — forms pre-fill from `defaults`, dropdowns from `options`
(via `values/index.ts`). There is no bundled fallback; the server is authoritative.
See `auto.example.toml` for the full schema. Built-in defaults reproduce prior
behavior, so a missing `auto.toml` changes nothing.

When adding a defaultable field or option list, update both `src/config.rs`
(struct + `Default`) and the frontend `PublicConfig` types in
`frontend/src/types/index.ts`.

## Adding a New Entity

1. Create `src/models/{entity}.rs` with Entity, Create, Update, WithRelations structs
2. Create `src/service/{entity}.rs` with CRUD functions
3. Create `src/api/{entity}.rs` with handler functions and utoipa annotations
4. Add route in `src/api/mod.rs` and register schemas in `src/openapi.rs`
5. Add migration in `/migrations/`
6. Add TypeScript types in `frontend/src/types/index.ts`
7. Add API namespace in `frontend/src/api/index.ts`
8. Add route in `frontend/src/router.ts`, views, and form components
