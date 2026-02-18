ALTER TABLE healthcheck ADD COLUMN retry INTEGER NOT NULL DEFAULT 0;
ALTER TABLE healthcheck ADD COLUMN retry_interval INTEGER NOT NULL DEFAULT 60;

-- JSON, x-www-form-urlencoded, XML
ALTER TABLE healthcheck ADD COLUMN request_body_encoding STRING NOT NULL DEFAULT 'JSON';
ALTER TABLE healthcheck ADD COLUMN request_body STRING; -- JSON

-- if these are non-null, the healthcheck uses HTTP basic authentication
ALTER TABLE healthcheck ADD COLUMN http_auth_user STRING; -- Basic Authentication username
ALTER TABLE healthcheck ADD COLUMN http_auth_pass STRING; -- Basic Authentication password
