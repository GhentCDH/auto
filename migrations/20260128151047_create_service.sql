-- a service can be shared by multiple applications,
-- it can be things like elasticsearch cluster,
-- load balancer, ...
CREATE TABLE service (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    repository_url TEXT,
    environment TEXT NOT NULL, -- 'dev', 'prd', ...
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT
);

CREATE TABLE application_service (
    application_id TEXT NOT NULL REFERENCES application(id) ON DELETE CASCADE,
    service_id TEXT NOT NULL REFERENCES service(id) ON DELETE CASCADE,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (application_id, service_id)
);

CREATE INDEX idx_application_service_app ON application_service(application_id);
CREATE INDEX idx_application_service_service ON application_service(service_id);
