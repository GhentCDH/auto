# Auto

A lightweight IT asset management tool for tracking applications, services, infrastructure, and their relationships. \
Built as a single binary with an embedded web interface.

<img width="100%" alt="Dashboard of Auto" style="max-width: 1000px;" src="https://github.com/user-attachments/assets/6c92330f-13f7-42ed-8334-ba8eb0981329" />

## Features

- **Applications** - Track apps with their environments, status, and repository links
- **Services** - Manage backend services and their infrastructure dependencies
- **Infrastructure** - Document servers, databases, and other infrastructure components
- **Domains** - Monitor domain registrations
- **People** - Keep track of team members and their responsibilities
- **Network Shares** - Document file shares and mount points
- **Tech Stacks** - Group applications by technology stack

All entities can be linked together with relationship metadata, making it easy to understand dependencies and ownership.

## Command line, TUI and agents

The `auto` binary (in [`auto-cli/`](./auto-cli)) is a client for a running auto server:

```bash
curl -fsSL https://raw.githubusercontent.com/GhentCDH/auto/main/scripts/install.sh | sh
auto login --url auto.example.org       # browser sign-in, stores a session
auto                                    # read-only TUI
auto list apps -F environment=prd       # tables on a terminal, JSON when piped
auto get infra vm-web-01                # by id, id prefix or exact name
auto schema applications                # fields, defaults, allowed values, links
auto create people name=Alice email=alice@example.org --dry-run
auto link applications wiki people alice contribution_type=maintainer
auto mcp                                # MCP server on stdio for agents
```

Fields and link relations are read from the server's OpenAPI document at
runtime, so the CLI does not need a release when the backend gains fields.
See [`docs/agents.md`](./docs/agents.md) for setting it up for Claude Code,
Claude Desktop and other agents.

## Quick Start

The app is built as a Rust backend at `src/` (with Axum as the web framework) with a Vue frontend at `frontend/`.

Configuration is done through environment variables, managed in an `.env` file.
The server reads environment variables and overwrites them with `.env`, or `dev.env` when present.

You can get started by copying `dev.env` to `.env`, and changing what you want.

### Database

Data is stored in an SQLite database. To create this database, choose a location for the database file, e.g. `data/data.db`.
Then configure that in the environment; e.g. `DATABASE_URL=sqlite://data/data.db`. \
The `create-db` just recipe then creates a database and performs migrations on it:

```sh
just create-db
```

### Development Server

```bash
just dev
```

This command starts up the frontend server at `http://localhost:5173`, which proxies the api requests
to a local dev-build of the backend.

It should automatically reload on changes.

### Building

Production builds are done with the `--release/-r` option for `cargo build`:

```bash
# Build and run
cargo build --release
./target/release/auto-server
```

This generates a single binary that has the frontend embedded in it.

Note that `cargo build --release` builds the frontend first (as defined in [`build.rs`](./build.rs))

## Development

This project uses [just](https://github.com/casey/just) as a command runner.

### Database

Requires `sqlx-cli` (`cargo install sqlx-cli`). \
It's a CLI that manages database creation, migrations, etc.

```bash
just create-db         # Create database and run migrations
just migrate           # Run pending migrations
just reset-db          # Reset database (destructive)
```

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
bun run dev            # Dev server with hot reload (proxies /api to :8080)
bun run build          # Production build
bun run format         # Format with Prettier
```

For frontend development, run both the backend (`cargo run`) and frontend dev server (`bun run dev`) simultaneously.

### Docker

```bash
just docker-build                    # Build image with tag auto:latest
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

**Frontend:** Vue 3, TypeScript, TailwindCSS, DaisyUI, Vite
