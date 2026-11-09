-- IPs a domain resolves to. `via` is 'A' for direct A/AAAA records, or 'CNAME'
-- when the domain is an alias and the IP comes from the end of the chain.
CREATE TABLE domain_ip (
    domain_id TEXT NOT NULL REFERENCES domain(id) ON DELETE CASCADE,
    ip TEXT NOT NULL,
    via TEXT NOT NULL CHECK (via IN ('A', 'CNAME')),
    cname TEXT,
    last_synced_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (domain_id, ip)
);

CREATE INDEX idx_domain_ip_ip ON domain_ip(ip);
