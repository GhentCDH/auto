-- Create host table for servers, VMs, Nomad jobs, containers
CREATE TABLE host (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    host_type TEXT NOT NULL, -- physical, vm, nomad, container
    hostname TEXT,
    ip_address TEXT,
    location TEXT,
    os TEXT,
    specs TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT
);

-- Junction table for application-host relationships
CREATE TABLE application_host (
    application_id TEXT NOT NULL REFERENCES application(id) ON DELETE CASCADE,
    host_id TEXT NOT NULL REFERENCES host(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'primary', -- primary, backup, staging, development
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (application_id, host_id)
);

CREATE INDEX idx_application_host_app ON application_host(application_id);
CREATE INDEX idx_application_host_host ON application_host(host_id);
