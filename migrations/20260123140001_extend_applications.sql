-- Extend application table with additional fields
ALTER TABLE application ADD COLUMN description TEXT;
ALTER TABLE application ADD COLUMN repository_url TEXT;
ALTER TABLE application ADD COLUMN status TEXT NOT NULL DEFAULT 'active';
ALTER TABLE application ADD COLUMN created_at TEXT NOT NULL DEFAULT (datetime('now'));
ALTER TABLE application ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));
ALTER TABLE application ADD COLUMN created_by TEXT;
