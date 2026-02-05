-- Healthcheck table for monitoring application and service endpoints
CREATE TABLE healthcheck (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,

    -- Target: XOR constraint (exactly one of application_id OR service_id)
    application_id TEXT REFERENCES application(id) ON DELETE CASCADE,
    service_id TEXT REFERENCES service(id) ON DELETE CASCADE,

    -- Domain FK (for constructing URL)
    domain_id TEXT NOT NULL REFERENCES domain(id) ON DELETE RESTRICT,

    -- Request config
    protocol TEXT NOT NULL DEFAULT 'https' CHECK (protocol IN ('http', 'https')),
    path TEXT NOT NULL DEFAULT '/',
    method TEXT NOT NULL DEFAULT 'GET' CHECK (method IN ('GET', 'HEAD', 'POST')),
    headers TEXT,  -- JSON: {"Host": "example.com"}

    -- Expected response
    expected_status INTEGER NOT NULL DEFAULT 200,
    expected_body TEXT,  -- Optional substring match

    -- Config
    timeout_seconds INTEGER NOT NULL DEFAULT 30,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    notes TEXT,

    -- Audit
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,

    -- XOR: exactly one of application_id OR service_id
    CHECK (
        (application_id IS NOT NULL AND service_id IS NULL) OR
        (application_id IS NULL AND service_id IS NOT NULL)
    )
);

CREATE INDEX idx_healthcheck_application ON healthcheck(application_id);
CREATE INDEX idx_healthcheck_service ON healthcheck(service_id);
CREATE INDEX idx_healthcheck_domain ON healthcheck(domain_id);
CREATE INDEX idx_healthcheck_enabled ON healthcheck(is_enabled);
