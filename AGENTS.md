# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Full-stack web application with a Rust backend (Axum) and Vue 3 frontend. The frontend is compiled and embedded into the Rust binary for single-binary deployment.

## Build Commands

### Backend (Rust)
```bash
cargo build          # Build backend
cargo run            # Run server (default: localhost:8080)
```

### Frontend (Vite/Bun)
```bash
cd frontend
bun install          # Install dependencies
bun run dev          # Start Vite dev server (proxies /api to localhost:6767)
bun run build        # Build to frontend-dist/ (embedded in backend)
bun run dev:build    # Watch mode build
bun run format       # Format with Prettier
```

### Full Development Workflow
1. Run `cargo run` for the backend
2. In another terminal, run `cd frontend && bun run dev` for hot-reload frontend development
3. Before deployment, run `cd frontend && bun run build` then rebuild the Rust binary

## Architecture

### Backend (`/src`)
- **main.rs** - Entry point, initializes tracing and starts server
- **lib.rs** - Core exports, AppState (contains SQLite pool + Config)
- **config.rs** - Environment configuration (DOMAIN, DATABASE_URL)
- **routes.rs** - Router setup, serves frontend assets with SPA fallback
- **error.rs** - Centralized error types with Axum response conversion
- **api/** - API endpoints (mounted at `/api/*`)
- **service/** - Business logic layer

### Frontend (`/frontend/src`)
- Vue 3 + TypeScript + Vue Router
- Tailwind CSS v4 + DaisyUI for styling
- Built assets go to `/frontend-dist/` and are embedded via `rust-embed`

### Database
- SQLite with sqlx (migrations in `/migrations/`)
- Migrations run automatically on AppState initialization

## Configuration

Environment variables (loaded from `.env`, falls back to `dev.env`):
- `DOMAIN` - Server bind address (default: `localhost:8080`)
- `DATABASE_URL` - SQLite connection (default: `sqlite://data/data.db`)
