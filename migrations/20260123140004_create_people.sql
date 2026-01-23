-- Create person table for developers, maintainers, support contacts
CREATE TABLE person (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE,
    role TEXT,
    department TEXT,
    phone TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT
);

-- Junction table for application-person relationships
CREATE TABLE application_person (
    application_id TEXT NOT NULL REFERENCES application(id) ON DELETE CASCADE,
    person_id TEXT NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    contribution_type TEXT NOT NULL DEFAULT 'developer', -- developer, maintainer, support, owner
    start_date TEXT,
    end_date TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (application_id, person_id)
);

CREATE INDEX idx_application_person_app ON application_person(application_id);
CREATE INDEX idx_application_person_person ON application_person(person_id);
