-- Track whether a healthcheck has local changes not yet synced to Kuma.
-- New healthchecks default to dirty (need initial sync).
-- Existing rows: dirty if no kuma_id (never synced), clean if kuma_id set.
ALTER TABLE healthcheck ADD COLUMN kuma_dirty INTEGER NOT NULL DEFAULT 1;

UPDATE healthcheck SET kuma_dirty = 0 WHERE kuma_id IS NOT NULL;
