PRAGMA foreign_keys=OFF;

-- Drop old domain-related tables and indexes
DROP TABLE IF EXISTS application_domain;
DROP TABLE IF EXISTS domain;
DROP INDEX IF EXISTS idx_application_domain_app;
DROP INDEX IF EXISTS idx_application_domain_domain;
DROP INDEX IF EXISTS idx_domain_name;

-- Create new domain table with XOR pattern
CREATE TABLE domain (
    id TEXT PRIMARY KEY,
    fqdn TEXT NOT NULL UNIQUE,
    registrar TEXT,
    dns_provider TEXT,
    expires_at TEXT,
    notes TEXT,
    target_application_id TEXT REFERENCES application(id) ON DELETE CASCADE,
    target_service_id TEXT REFERENCES service(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    CHECK (
        (target_application_id IS NOT NULL AND target_service_id IS NULL) OR
        (target_application_id IS NULL AND target_service_id IS NOT NULL)
    )
);

CREATE INDEX idx_domain_application ON domain(target_application_id);
CREATE INDEX idx_domain_service ON domain(target_service_id);
CREATE INDEX idx_domain_fqdn ON domain(fqdn);

-- Junction table for application-domain relationships
CREATE TABLE application_domain (
    application_id TEXT NOT NULL REFERENCES application(id) ON DELETE CASCADE,
    domain_id TEXT NOT NULL REFERENCES domain(id) ON DELETE CASCADE,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (application_id, domain_id)
);

CREATE INDEX idx_application_domain_app ON application_domain(application_id);
CREATE INDEX idx_application_domain_domain ON application_domain(domain_id);

PRAGMA foreign_keys=ON;
