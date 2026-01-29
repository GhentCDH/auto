# Auto

A lightweight IT asset management tool for tracking applications, services, infrastructure, and their relationships. Built as a single binary with an embedded web interface.

![AUTO](https://github.com/user-attachments/assets/d0688b2a-306e-420f-a718-21f5c00a2cdd)

## Features

- **Applications** - Track apps with their environments, status, and repository links
- **Services** - Manage backend services and their infrastructure dependencies
- **Infrastructure** - Document servers, databases, and other infrastructure components
- **Domains** - Monitor domain registrations
- **People** - Keep track of team members and their responsibilities
- **Network Shares** - Document file shares and mount points
- **Tech Stacks** - Group applications by technology stack

All entities can be linked together with relationship metadata, making it easy to understand dependencies and ownership.

## Quick Start

```bash
# Build and run
cargo build --release
./target/release/auto

# Or for development
cargo run
```

The server starts at `localhost:8080` by default. Configure via environment variables or `.env` file:

```env
DOMAIN=localhost:8080
DATABASE_URL=sqlite://data/data.db
```

For logging output, set `RUST_LOG`:

```bash
RUST_LOG=info cargo run
```

## Development

This project uses [just](https://github.com/casey/just) as a command runner.

### Database

```bash
just create-db         # Create database and run migrations
just migrate           # Run pending migrations
just reset-db          # Reset database (destructive)
just prepare           # Generate sqlx query cache for offline builds
```

Requires `sqlx-cli` (`cargo install sqlx-cli`).

### Backend

```bash
cargo run              # Run server
just watch             # Run with auto-reload on changes (requires cargo-watch)
cargo check            # Type check
cargo test             # Run tests
```

### Frontend

```bash
cd frontend
bun install            # Install dependencies
bun run dev            # Dev server with hot reload (proxies /api to :6767)
bun run build          # Production build
bun run format         # Format with Prettier
```

For frontend development, run both the backend (`cargo run`) and frontend dev server (`bun run dev`) simultaneously.

### Docker

```bash
just docker-build                    # Build image
just docker-run                      # Run with defaults (dev.env, port 8080)
just docker-run env-file=prod.env    # Run with custom env file
```

## Project Structure

```
├── src/
│   ├── main.rs          # Entry point, tracing setup
│   ├── lib.rs           # AppState, exports
│   ├── config.rs        # Environment configuration
│   ├── routes.rs        # Router, static file serving
│   ├── error.rs         # Error types
│   ├── api/             # REST API endpoints
│   │   ├── mod.rs       # Route registration
│   │   ├── applications.rs
│   │   ├── services.rs
│   │   └── ...
│   └── service/         # Business logic
│
├── frontend/
│   └── src/
│       ├── api/         # API client functions
│       ├── components/  # Reusable Vue components
│       ├── views/       # Page components
│       ├── types/       # TypeScript interfaces
│       └── router.ts    # Vue Router config
│
├── migrations/          # SQLx migrations (auto-run on startup)
└── frontend-dist/       # Built frontend (embedded in binary)
```

### Tech Stack

**Backend:** Rust, Axum, SQLx, SQLite, rust-embed

**Frontend:** Vue 3, TypeScript, Tailwind CSS, DaisyUI, Vite
