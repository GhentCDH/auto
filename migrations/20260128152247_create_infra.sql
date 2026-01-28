CREATE TABLE infra (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    type TEXT NOT NULL, -- nomad cluster, server, ...
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT
);

CREATE TABLE application_infra (
    application_id TEXT NOT NULL REFERENCES application(id) ON DELETE CASCADE,
    infra_id TEXT NOT NULL REFERENCES infra(id) ON DELETE CASCADE,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (application_id, infra_id)
);

CREATE INDEX idx_application_infra_app ON application_infra(application_id);
CREATE INDEX idx_application_infra_infra ON application_infra(infra_id);

CREATE TABLE service_infra (
    service_id TEXT NOT NULL REFERENCES service(id) ON DELETE CASCADE,
    infra_id TEXT NOT NULL REFERENCES infra(id) ON DELETE CASCADE,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (service_id, infra_id)
);

CREATE INDEX idx_service_infra_app ON service_infra(service_id);
CREATE INDEX idx_service_infra_infra ON service_infra(infra_id);

