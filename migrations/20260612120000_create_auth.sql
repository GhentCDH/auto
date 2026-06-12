-- Authentication: users, credentials, roles, sessions and password tokens.
-- A "user" is the internal identity; credentials (password / oidc) are how
-- they prove it. One user may have several credentials attached.

CREATE TABLE users (
    id          TEXT PRIMARY KEY,
    username    TEXT NOT NULL UNIQUE,
    email       TEXT UNIQUE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Authorization source of truth for every user, regardless of auth method.
CREATE TABLE user_roles (
    user_id  TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    role     TEXT NOT NULL CHECK (role IN ('admin', 'editor', 'viewer'))
);

-- Password credentials. Row is absent until the user sets a password via a
-- setup token, so an admin-created account has no password until first login.
CREATE TABLE password_credentials (
    user_id       TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL,
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- One row per linked OIDC identity. A user can link identities from multiple
-- issuers; (issuer, sub) is globally unique.
CREATE TABLE oidc_identities (
    user_id  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    issuer   TEXT NOT NULL,
    sub      TEXT NOT NULL,
    PRIMARY KEY (issuer, sub)
);
CREATE INDEX idx_oidc_identities_user ON oidc_identities(user_id);

-- Server-side sessions. The client only ever holds the opaque `id` token.
-- `role` is denormalised here so request auth needs a single lookup.
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    method      TEXT NOT NULL,                       -- "password" or "oidc:<issuer>"
    role        TEXT NOT NULL,
    expires_at  TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_sessions_user ON sessions(user_id);

-- Secret links for setting a password: used both for first-login setup and for
-- admin-fulfilled resets. The token is the random value embedded in the link.
CREATE TABLE password_setup_tokens (
    token       TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TEXT NOT NULL,
    used        INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_password_setup_tokens_user ON password_setup_tokens(user_id);

-- Forgotten-password requests. A logged-out user submits one; admins see the
-- pending list and generate a setup link to share back.
CREATE TABLE password_reset_requests (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'fulfilled')),
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_password_reset_requests_status ON password_reset_requests(status);

-- An OIDC identity awaiting confirmation that the user owns a matching password
-- account. Cleared once the link is completed (or it expires).
CREATE TABLE pending_oidc_identities (
    token       TEXT PRIMARY KEY,
    issuer      TEXT NOT NULL,
    sub         TEXT NOT NULL,
    email       TEXT,
    expires_at  TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
