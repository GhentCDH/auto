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

Environment variables (loaded from `.env`, falls back to `dev.env`):

- `DOMAIN` - Server bind address (default: `0.0.0.0:8080`)
- `DATABASE_URL` - SQLite connection (default: `sqlite://data/data.db`)
- `KUMA_URL` - Uptime Kuma instance URL
- `KUMA_USERNAME` / `KUMA_PASSWORD` - Kuma authentication credentials

## Adding a New Entity

1. Create `src/models/{entity}.rs` with Entity, Create, Update, WithRelations structs
2. Create `src/service/{entity}.rs` with CRUD functions
3. Create `src/api/{entity}.rs` with handler functions and utoipa annotations
4. Add route in `src/api/mod.rs` and register schemas in `src/openapi.rs`
5. Add migration in `/migrations/`
6. Add TypeScript types in `frontend/src/types/index.ts`
7. Add API namespace in `frontend/src/api/index.ts`
8. Add route in `frontend/src/router.ts`, views, and form components
