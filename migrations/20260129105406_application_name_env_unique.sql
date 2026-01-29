PRAGMA foreign_keys=OFF;

CREATE TABLE application_new (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    repository_url TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    environment TEXT NOT NULL DEFAULT 'prd',
    url TEXT,
    UNIQUE(name, environment)
);

INSERT INTO application_new (id, name, description, repository_url, status, created_at, updated_at, created_by, environment, url)
SELECT id, name, description, repository_url, status, created_at, updated_at, created_by, environment, url
FROM application;

DROP TABLE application;

ALTER TABLE application_new RENAME TO application;

PRAGMA foreign_keys=ON;
