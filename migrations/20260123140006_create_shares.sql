-- Create network_share table for SMB/NFS shares
CREATE TABLE network_share (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    path TEXT NOT NULL,
    share_type TEXT NOT NULL DEFAULT 'smb', -- smb, nfs, cifs
    server TEXT,
    purpose TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT
);

-- Junction table for application-share relationships
CREATE TABLE application_network_share (
    application_id TEXT NOT NULL REFERENCES application(id) ON DELETE CASCADE,
    network_share_id TEXT NOT NULL REFERENCES network_share(id) ON DELETE CASCADE,
    usage TEXT, -- config, data, logs, backup
    mount_point TEXT,
    permissions TEXT, -- read, write, read-write
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (application_id, network_share_id)
);

CREATE INDEX idx_application_share_app ON application_network_share(application_id);
CREATE INDEX idx_application_share_share ON application_network_share(network_share_id);
