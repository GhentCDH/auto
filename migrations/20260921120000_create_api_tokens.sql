-- Long-lived API tokens for non-browser clients (for example the auto TUI).
-- Only the SHA-256 hash of the token is stored; the plaintext is shown once at
-- creation. Tokens carry the owning user's live role at request time.

CREATE TABLE api_tokens (
    id           TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    token_hash   TEXT NOT NULL UNIQUE,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at   TEXT,
    last_used_at TEXT
);

CREATE INDEX idx_api_tokens_user ON api_tokens(user_id);
