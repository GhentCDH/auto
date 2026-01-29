-- remove domain ssl fields
ALTER TABLE domain DROP COLUMN ssl_expires_at; 
ALTER TABLE domain DROP COLUMN ssl_issuer; 
