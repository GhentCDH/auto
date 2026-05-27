//! Live DNS resolution for domains.
//!
//! Records are **never** persisted — they are resolved on demand via the system
//! resolver and cached in memory for a short TTL ([`DNS_CACHE_TTL`]) so repeated
//! page loads don't hammer the resolver. This mirrors the action-endpoint style
//! used by healthcheck execution rather than the SSE uptime stream.

use std::time::{Duration, Instant};

use chrono::Utc;
use futures::{StreamExt, stream};
use hickory_resolver::net::{DnsError, NetError};
use hickory_resolver::proto::rr::{RData, RecordType};

use crate::models::{DnsLookup, DnsRecord};
use crate::{AppState, Error, Result};

/// How long a resolved record set stays fresh in the in-memory cache.
const DNS_CACHE_TTL: Duration = Duration::from_secs(60);

/// Record types we surface on domain pages, in display order.
const QUERY_TYPES: [RecordType; 6] = [
    RecordType::A,
    RecordType::AAAA,
    RecordType::CNAME,
    RecordType::MX,
    RecordType::TXT,
    RecordType::NS,
];

/// Max concurrent per-domain resolutions during a bulk lookup.
const BULK_CONCURRENCY: usize = 8;

/// Resolve all queried record types for a single FQDN, using the cache when fresh.
pub async fn lookup(state: &AppState, fqdn: &str) -> Result<DnsLookup> {
    // Fast path: serve from cache if the entry hasn't expired.
    if let Some((at, resolved_at, records)) = state.dns_cache.read().await.get(fqdn) {
        if at.elapsed() < DNS_CACHE_TTL {
            return Ok(DnsLookup {
                fqdn: fqdn.to_string(),
                records: records.clone(),
                resolved_at: resolved_at.clone(),
                error: None,
            });
        }
    }

    let records = resolve_all_types(state, fqdn).await?;
    let resolved_at = Utc::now().to_rfc3339();

    state.dns_cache.write().await.insert(
        fqdn.to_string(),
        (Instant::now(), resolved_at.clone(), records.clone()),
    );

    Ok(DnsLookup {
        fqdn: fqdn.to_string(),
        records,
        resolved_at,
        error: None,
    })
}

/// Resolve DNS records for every known domain with bounded concurrency.
///
/// A single domain failing to resolve does not abort the batch — its entry comes
/// back with empty `records` and a populated `error` field instead.
pub async fn lookup_all(state: &AppState) -> Result<Vec<DnsLookup>> {
    let fqdns: Vec<String> = sqlx::query_scalar("SELECT fqdn FROM domain ORDER BY fqdn")
        .fetch_all(&state.pool)
        .await?;

    let lookups = stream::iter(fqdns)
        .map(|fqdn| async move {
            match lookup(state, &fqdn).await {
                Ok(result) => result,
                Err(e) => DnsLookup {
                    fqdn,
                    records: Vec::new(),
                    resolved_at: Utc::now().to_rfc3339(),
                    error: Some(e.to_string()),
                },
            }
        })
        .buffer_unordered(BULK_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    Ok(lookups)
}

/// Query every record type concurrently and flatten the answers into [`DnsRecord`]s.
async fn resolve_all_types(state: &AppState, fqdn: &str) -> Result<Vec<DnsRecord>> {
    let queries = QUERY_TYPES
        .iter()
        .map(|&rtype| query_type(state, fqdn, rtype));

    let per_type = futures::future::try_join_all(queries).await?;
    Ok(per_type.into_iter().flatten().collect())
}

/// Resolve a single record type. "No records found" (incl. NXDOMAIN) yields an
/// empty Vec rather than an error — a domain may legitimately lack MX, AAAA, etc.
/// Only transport/resolver failures propagate as [`Error::DnsError`].
async fn query_type(state: &AppState, fqdn: &str, rtype: RecordType) -> Result<Vec<DnsRecord>> {
    match state.resolver.lookup(fqdn, rtype).await {
        Ok(lookup) => Ok(lookup
            .answers()
            .iter()
            // A CNAME chase returns the CNAME chain *and* the target's A/AAAA in one
            // answer; keep only records of the type we asked for so the CNAME isn't
            // also emitted by the A query (it comes from the dedicated CNAME query).
            .filter(|record| record.record_type() == rtype)
            .map(|record| {
                let (value, priority) = match &record.data {
                    RData::MX(mx) => (mx.exchange.to_string(), Some(mx.preference)),
                    other => (other.to_string(), None),
                };
                DnsRecord {
                    record_type: record.record_type().to_string(),
                    value: trim_trailing_dot(&value),
                    ttl: record.ttl,
                    priority,
                }
            })
            .collect()),
        // A domain may legitimately lack a given record type — surface as empty, not an error.
        Err(NetError::Dns(DnsError::NoRecordsFound(_))) => Ok(Vec::new()),
        Err(e) => Err(Error::DnsError(format!(
            "{rtype} lookup for {fqdn} failed: {e}"
        ))),
    }
}

/// DNS names carry a trailing dot (FQDN root); strip it for display.
fn trim_trailing_dot(value: &str) -> String {
    value.strip_suffix('.').unwrap_or(value).to_string()
}
