-- Create client table for who applications were built for
CREATE TABLE client (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    contact_name TEXT,
    contact_email TEXT,
    department TEXT,
    phone TEXT,
    address TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT
);

-- Junction table for application-client relationships
CREATE TABLE application_client (
    application_id TEXT NOT NULL REFERENCES application(id) ON DELETE CASCADE,
    client_id TEXT NOT NULL REFERENCES client(id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL DEFAULT 'customer', -- customer, sponsor, internal
    contract_ref TEXT,
    start_date TEXT,
    end_date TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (application_id, client_id)
);

CREATE INDEX idx_application_client_app ON application_client(application_id);
CREATE INDEX idx_application_client_client ON application_client(client_id);
