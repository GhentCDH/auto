-- Stack table for technology tracking (languages, frameworks, etc.)
CREATE TABLE stack (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Junction table for application-stack relationships
CREATE TABLE application_stack (
    application_id TEXT NOT NULL REFERENCES application(id) ON DELETE CASCADE,
    stack_id TEXT NOT NULL REFERENCES stack(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (application_id, stack_id)
);

-- Indexes for efficient queries
CREATE INDEX idx_application_stack_app ON application_stack(application_id);
CREATE INDEX idx_application_stack_stack ON application_stack(stack_id);
