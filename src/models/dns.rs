use serde::Serialize;
use utoipa::ToSchema;

/// A single DNS record resolved live for a domain's FQDN.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DnsRecord {
    /// Record type: "A", "AAAA", "CNAME", "MX", "TXT", "NS".
    pub record_type: String,
    /// Formatted value: IP address, target host, or text content.
    pub value: String,
    /// Time-to-live in seconds, as reported by the resolver.
    pub ttl: u32,
    /// Preference value — only populated for MX records.
    pub priority: Option<u16>,
    /// Set when this record's value (an A/AAAA IP) matches a known infra IP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infra: Option<InfraMatch>,
}

/// A backlink from a DNS A/AAAA record to infrastructure tracked in Auto,
/// when the resolved IP matches a stored `infra_ip`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InfraMatch {
    pub id: String,
    pub name: String,
}

/// The result of a live DNS lookup for one FQDN.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DnsLookup {
    /// The fully-qualified domain name that was queried.
    pub fqdn: String,
    /// All resolved records across the queried types.
    pub records: Vec<DnsRecord>,
    /// RFC3339 timestamp of when this lookup was resolved (cache fill time).
    pub resolved_at: String,
    /// Set when this domain failed to resolve during a bulk lookup; `None` on success.
    /// Lets the all-domains endpoint report partial failures without aborting the batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
