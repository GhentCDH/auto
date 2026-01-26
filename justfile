export DATABASE_URL := "sqlite://data/data.db"

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
    if [ ! -f "data/data.db" ]; then
        echo "Error: Database does not exist at data/data.db"
        echo "Create it with: just create-db"
        exit 1
    fi

# Create the database and run migrations
create-db: _require-sqlx
    cargo sqlx database create
    cargo sqlx migrate run

migrate: _require-sqlx
    cargo sqlx migrate run

reset-db: _require-sqlx
    cargo sqlx database reset -y

# Generate .sqlx query cache for offline compilation
prepare: _require-sqlx _require-db
    cargo sqlx prepare
