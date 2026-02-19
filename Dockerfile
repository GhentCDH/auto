# Stage 1: Build frontend
FROM oven/bun:1 AS frontend-builder

WORKDIR /app/frontend

# Copy package files first for better layer caching
COPY frontend/package.json frontend/bun.lock* ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy frontend source (node_modules ignored in .dockerignore)
COPY frontend/ .

# Build frontend (outputs to ../frontend-dist/)
RUN bun run build

# Stage 2: Build Rust backend

FROM lukemathwalker/cargo-chef:0.1.73-rust-1.93-alpine3.22 as chef

WORKDIR /app

FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig curl

COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY --from=frontend-builder /app/frontend-dist /app/frontend-dist

RUN cargo build --release
RUN mv target/release/auto app

FROM alpine:3.23 AS runtime

RUN apk add --no-cache tini

WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/

ENTRYPOINT ["/sbin/tini", "--", "/usr/local/bin/app"]
