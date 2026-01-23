-- Create note table for documentation, changelogs, issues
CREATE TABLE note (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL, -- application, host, domain, person, client, network_share
    entity_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    note_type TEXT NOT NULL DEFAULT 'general', -- general, documentation, changelog, issue, link
    url TEXT,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT
);

CREATE INDEX idx_note_entity ON note(entity_type, entity_id);
CREATE INDEX idx_note_type ON note(note_type);
