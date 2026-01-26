-- Create domain table for DNS records and SSL info
CREATE TABLE domain (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    registrar TEXT,
    dns_provider TEXT,
    expires_at TEXT,
    ssl_expires_at TEXT,
    ssl_issuer TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT
);

-- Junction table for application-domain relationships
CREATE TABLE application_domain (
    application_id TEXT NOT NULL REFERENCES application(id) ON DELETE CASCADE,
    domain_id TEXT NOT NULL REFERENCES domain(id) ON DELETE CASCADE,
    record_type TEXT NOT NULL DEFAULT 'A', -- A, CNAME, etc.
    target TEXT,
    target_host_id TEXT REFERENCES host(id) ON DELETE SET NULL,
    is_primary INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (application_id, domain_id),
    CHECK (target_host_id IS NULL OR target IS NULL)
);

CREATE INDEX idx_application_domain_app ON application_domain(application_id);
CREATE INDEX idx_application_domain_domain ON application_domain(domain_id);
