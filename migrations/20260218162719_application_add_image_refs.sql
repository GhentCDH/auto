-- built docker images references (dockerhub, ghcr, ...)
ALTER TABLE application ADD COLUMN image_refs TEXT;
ALTER TABLE service ADD COLUMN image_refs TEXT;
