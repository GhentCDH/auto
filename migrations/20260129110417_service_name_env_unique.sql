PRAGMA foreign_keys=OFF;

CREATE TABLE service_new (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    repository_url TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    environment TEXT NOT NULL default 'prd', -- 'dev', 'prd', ...
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    UNIQUE(name, environment)
);

INSERT INTO service_new (id, name, description, repository_url, environment, status, created_at, updated_at, created_by)
SELECT id, name, description, repository_url, environment, status, created_at, updated_at, created_by
FROM service;

DROP TABLE service;

ALTER TABLE service_new RENAME TO service;

PRAGMA foreign_keys=ON;
