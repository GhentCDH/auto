-- In-flight CLI/browser login flows. A non-browser client (the TUI) asks for a
-- flow and a loopback port; the browser authorizes it after logging in, and the
-- client exchanges the one-time code for a normal session token.

CREATE TABLE cli_login_flows (
    id          TEXT PRIMARY KEY,
    port        INTEGER NOT NULL,
    user_id     TEXT REFERENCES users(id) ON DELETE CASCADE,
    method      TEXT,
    code        TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at  TEXT NOT NULL,
    used        INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_cli_login_flows_expires ON cli_login_flows(expires_at);
