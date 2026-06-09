-- Track whether a healthcheck has notifications enabled or not
ALTER TABLE healthcheck ADD COLUMN notifications INTEGER NOT NULL DEFAULT 0;
