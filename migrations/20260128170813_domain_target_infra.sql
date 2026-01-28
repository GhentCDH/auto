-- Rename target_host_id to target_infra_id in application_domain table
-- SQLite requires table recreation to change foreign key references

-- Create new table with target_infra_id
CREATE TABLE application_domain_new (
    application_id TEXT NOT NULL REFERENCES application(id) ON DELETE CASCADE,
    domain_id TEXT NOT NULL REFERENCES domain(id) ON DELETE CASCADE,
    record_type TEXT NOT NULL DEFAULT 'A',
    target TEXT,
    target_infra_id TEXT REFERENCES infra(id) ON DELETE SET NULL,
    is_primary INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (application_id, domain_id),
    CHECK (target_infra_id IS NULL OR target IS NULL)
);

-- Copy data from old table (target_host_id becomes target_infra_id, but will be NULL since hosts are removed)
INSERT INTO application_domain_new (application_id, domain_id, record_type, target, target_infra_id, is_primary, notes, created_at)
SELECT application_id, domain_id, record_type, target, NULL, is_primary, notes, created_at
FROM application_domain;

-- Drop old table
DROP TABLE application_domain;

-- Rename new table
ALTER TABLE application_domain_new RENAME TO application_domain;

-- Recreate indexes
CREATE INDEX idx_application_domain_app ON application_domain(application_id);
CREATE INDEX idx_application_domain_domain ON application_domain(domain_id);
