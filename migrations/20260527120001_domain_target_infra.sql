-- Add target_infra_id to domain and widen the target XOR to 3-way
-- (application | service | infra). SQLite cannot ALTER a CHECK constraint, so
-- the table is recreated.
--
-- domain is referenced by application_domain (CASCADE), healthcheck (RESTRICT)
-- and service_domain (SET NULL). With foreign_keys ON, DROP TABLE would fire an
-- implicit DELETE against those (RESTRICT even fails). This migration is run on a
-- connection with foreign_keys disabled (see AppState::new) so the drop/rename is
-- a pure schema swap — children keep referencing the same ids, still present in
-- the recreated table.

CREATE TABLE domain_new (
    id TEXT PRIMARY KEY,
    fqdn TEXT NOT NULL UNIQUE,
    registrar TEXT,
    dns_provider TEXT,
    expires_at TEXT,
    notes TEXT,
    target_application_id TEXT REFERENCES application(id) ON DELETE CASCADE,
    target_service_id TEXT REFERENCES service(id) ON DELETE CASCADE,
    target_infra_id TEXT REFERENCES infra(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    CHECK (
        (target_application_id IS NOT NULL AND target_service_id IS NULL AND target_infra_id IS NULL) OR
        (target_application_id IS NULL AND target_service_id IS NOT NULL AND target_infra_id IS NULL) OR
        (target_application_id IS NULL AND target_service_id IS NULL AND target_infra_id IS NOT NULL)
    )
);

INSERT INTO domain_new (
    id, fqdn, registrar, dns_provider, expires_at, notes,
    target_application_id, target_service_id, created_at, updated_at, created_by
)
SELECT
    id, fqdn, registrar, dns_provider, expires_at, notes,
    target_application_id, target_service_id, created_at, updated_at, created_by
FROM domain;

DROP TABLE domain;

ALTER TABLE domain_new RENAME TO domain;

CREATE INDEX idx_domain_application ON domain(target_application_id);
CREATE INDEX idx_domain_service ON domain(target_service_id);
CREATE INDEX idx_domain_infra ON domain(target_infra_id);
CREATE INDEX idx_domain_fqdn ON domain(fqdn);
