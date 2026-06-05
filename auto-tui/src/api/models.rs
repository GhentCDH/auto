//! Deserialize-only mirrors of the backend wire types.
//!
//! Source of truth lives in the backend crate (`src/models/`, `src/service/`).
//! Entity list/detail bodies are kept as `serde_json::Value` and rendered
//! generically, so only the typed responses used directly by the UI are
//! mirrored here.

use std::collections::HashMap;

use serde::Deserialize;

/// Mirror of `PaginatedResponse<T>` (backend `src/models/mod.rs`).
#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Mirror of `DashboardStats` (backend `src/service/dashboard.rs`).
#[derive(Debug, Clone, Deserialize)]
pub struct DashboardStats {
    pub applications: EntityStats,
    pub services: EntityStats,
    pub infra: EntityStats,
    pub domains: EntityStats,
    pub people: EntityStats,
    pub network_shares: EntityStats,
    pub notes: i64,
    pub expiring_domains: Vec<ExpiringDomain>,
    pub healthchecks: HealthcheckStats,
    pub recent_activity: Vec<RecentActivity>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntityStats {
    pub total: i64,
    pub active: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthcheckStats {
    pub total: i64,
    pub enabled: i64,
    pub kuma_dirty: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExpiringDomain {
    pub id: String,
    pub fqdn: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecentActivity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub updated_at: String,
}

/// Mirror of `SearchResults` (backend `src/service/search.rs`).
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResults {
    pub applications: Vec<SearchResult>,
    pub services: Vec<SearchResult>,
    pub infra: Vec<SearchResult>,
    pub domains: Vec<SearchResult>,
    pub people: Vec<SearchResult>,
    pub network_shares: Vec<SearchResult>,
    pub stacks: Vec<SearchResult>,
    pub healthchecks: Vec<SearchResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
}

/// Mirror of `DnsLookup` (backend `src/models/dns.rs`).
#[derive(Debug, Clone, Deserialize)]
pub struct DnsLookup {
    pub fqdn: String,
    pub records: Vec<DnsRecord>,
    pub resolved_at: String,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DnsRecord {
    pub record_type: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
    #[serde(default)]
    pub infra: Option<InfraMatch>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InfraMatch {
    pub id: String,
    pub name: String,
}

/// Mirror of `HealthcheckExecuteResult` (backend `src/models/healthcheck.rs`).
#[derive(Debug, Clone, Deserialize)]
pub struct HealthcheckExecuteResult {
    pub healthcheck_id: String,
    pub url: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_time_ms: u64,
    pub body_match: Option<bool>,
    pub error: Option<String>,
    pub executed_at: String,
}

/// Mirror of `HeartbeatEntry` (backend `src/models/uptime.rs`).
///
/// Status codes: 1 = up, 0 = down, 2 = pending, 3 = maintenance.
#[derive(Debug, Clone, Deserialize)]
pub struct HeartbeatEntry {
    pub status: i32,
    pub time: String,
    pub ping: Option<i32>,
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonitorUptime {
    pub kuma_id: i32,
    pub heartbeats: Vec<HeartbeatEntry>,
}

/// Mirror of `UptimeEvent` (backend `src/models/uptime.rs`), as sent over
/// the `/healthchecks/uptime/stream` SSE endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UptimeEvent {
    Snapshot {
        monitors: HashMap<i32, MonitorUptime>,
    },
    Update {
        kuma_id: i32,
        entry: HeartbeatEntry,
    },
}
