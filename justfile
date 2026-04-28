alias d := dev

set dotenv-load

export RUST_LOG := "debug,tower_http=info,h2=info,sqlx=info"

# Check if sqlx-cli is installed, provide install instructions if not
_require-sqlx:
    #!/usr/bin/env bash
    if ! command -v cargo-sqlx &> /dev/null && ! cargo sqlx --version &> /dev/null; then
        echo "Error: sqlx-cli is not installed"
        echo -e "Install it with: \x1b[1;33mcargo install sqlx-cli\x1b[0m"
        exit 1
    fi

# Check if database exists, provide instructions if not
_require-db:
    #!/usr/bin/env bash
    db_path="${DATABASE_URL#sqlite://}"
    if [ ! -f "$db_path" ]; then
        echo "Error: Database does not exist at ${db_path}"
        echo "Create it with: just create-db"
        exit 1
    fi

# Create the database and run migrations
create-db: _require-sqlx
    @[ -d "$(dirname "${DATABASE_URL#sqlite://}")" ] || printf "\x1b[31mThe directory for your database doesn't exist\x1b[0m\n"
    cargo sqlx database create
    cargo sqlx migrate run

migrate: _require-sqlx _require-db
    cargo sqlx migrate run

reset-db: _require-sqlx
    cargo sqlx database reset -y

dev:
    #!/usr/bin/env bash
    set -euo pipefail

    # Kill all background jobs when the script exits (Ctrl+C or error)
    trap 'echo "Stopping..."; kill $(jobs -p) 2>/dev/null; wait' EXIT

    echo "→ Starting Rust backend (cargo run)..."
    cargo run &

    echo "→ Starting Vite dev server (bun run dev)..."
    (cd frontend && bun run dev) &

    wait

watch:
    @cargo watch -x run

docker-build:
    docker build -t auto:latest .

docker-run env-file="dev.env" port="8080":
    docker run --env-file {{env-file}} -v ./data:/app/data -p {{port}}:{{port}} auto:latest

[working-directory: "frontend"]
watch-frontend:
    bun run watch
