-- Tracks IP addresses associated with infrastructure.
-- IPs are either resolved from the infra's domain (source = 'domain', refreshed
-- lazily when stale) or assigned manually (source = 'manual'). Not a cache of
-- live DNS — these are stored so we can match DNS A/AAAA records back to infra.
CREATE TABLE infra_ip (
    infra_id TEXT NOT NULL REFERENCES infra(id) ON DELETE CASCADE,
    ip TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'domain', -- 'domain' | 'manual'
    last_synced_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (infra_id, ip)
);

-- Fast IP -> infra lookups when matching DNS records.
CREATE INDEX idx_infra_ip_ip ON infra_ip(ip);
CREATE INDEX idx_infra_ip_infra ON infra_ip(infra_id);
