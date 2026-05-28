//! Infra IP tracking and link reconciliation.
//!
//! Link reconciliation refers to the process of linking infra to applications and services
//! when there is an IP match between a domain that's linked to said application or service and
//! the tracked IP of the infrastructure.

use std::collections::HashMap;

use chrono::{NaiveDateTime, Utc};

use crate::service::{application, dns, service as service_svc};
use crate::{AppState, Result};

/// Treat A/AAAA record values as comparable IP strings (IPv6 lowercased).
fn normalize_ip(value: &str) -> String {
    value.trim().to_lowercase()
}

/// Resolve and store the domain-sourced IPs for infra that have a domain target.
///
/// `infra_id = Some(_)` syncs one infra; `None` syncs all infra with a domain IP source.
/// A domain that fails to resolve is skipped (its previously stored IPs are kept)
/// rather than wiped. After syncing, link reconciliation runs.
pub async fn sync_infra_ips(state: &AppState, infra_id: Option<&str>) -> Result<()> {
    // (infra_id, fqdn) for each domain that targets an infra.
    let targets: Vec<(String, String)> = if let Some(id) = infra_id {
        sqlx::query_as("SELECT target_infra_id, fqdn FROM domain WHERE target_infra_id = ?1")
            .bind(id)
            .fetch_all(&state.pool)
            .await?
    } else {
        sqlx::query_as("SELECT target_infra_id, fqdn FROM domain WHERE target_infra_id IS NOT NULL")
            .fetch_all(&state.pool)
            .await?
    };

    for (infra_id, fqdn) in targets {
        let lookup = match dns::lookup(state, &fqdn).await {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!("infra IP sync: {fqdn} failed to resolve, keeping old IPs: {e}");
                continue;
            }
        };
        let ips: Vec<String> = lookup
            .records
            .iter()
            .filter(|r| r.record_type == "A" || r.record_type == "AAAA")
            .map(|r| normalize_ip(&r.value))
            .collect();

        // Replace only the domain-sourced rows; manual IPs are untouched.
        sqlx::query("DELETE FROM infra_ip WHERE infra_id = ?1 AND source = 'domain'")
            .bind(&infra_id)
            .execute(&state.pool)
            .await?;
        for ip in ips {
            sqlx::query(
                r#"
                INSERT INTO infra_ip (infra_id, ip, source, last_synced_at)
                VALUES (?1, ?2, 'domain', datetime('now'))
                ON CONFLICT (infra_id, ip) DO NOTHING
                "#,
            )
            .bind(&infra_id)
            .bind(&ip)
            .execute(&state.pool)
            .await?;
        }
    }

    reconcile_infra_links(state, None).await
}

/// Re-resolve an infra's IPs only if they're stale (older than the configured
/// interval) or never resolved. No-op for infra without a domain target.
pub async fn ensure_fresh_infra_ips(state: &AppState, infra_id: &str) -> Result<()> {
    let has_domain: Option<String> =
        sqlx::query_scalar("SELECT fqdn FROM domain WHERE target_infra_id = ?1 LIMIT 1")
            .bind(infra_id)
            .fetch_optional(&state.pool)
            .await?;
    if has_domain.is_none() {
        return Ok(());
    }

    let newest: Option<String> = sqlx::query_scalar(
        "SELECT MAX(last_synced_at) FROM infra_ip WHERE infra_id = ?1 AND source = 'domain'",
    )
    .bind(infra_id)
    .fetch_one(&state.pool)
    .await?;

    let stale = match newest {
        None => true, // has a domain but nothing resolved yet
        Some(ts) => is_older_than_days(&ts, state.config.infra_ip_refresh_days),
    };
    if stale {
        sync_infra_ips(state, Some(infra_id)).await?;
    }
    Ok(())
}

/// SQLite `datetime('now')` strings are `"YYYY-MM-DD HH:MM:SS"` in UTC.
fn is_older_than_days(ts: &str, days: u64) -> bool {
    match NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S") {
        Ok(parsed) => {
            let age = Utc::now().naive_utc() - parsed;
            age > chrono::Duration::days(days as i64)
        }
        // Unparseable timestamp → treat as stale so it gets refreshed.
        Err(_) => true,
    }
}

/// Add-only reconciliation: for every app/service that has a domain resolving to
/// a known infra IP, ensure an `application_infra`/`service_infra` link exists
/// with note `through <ip>`. Never removes links. `domain_id = Some(_)` scopes to
/// one domain (used by the live triggers).
pub async fn reconcile_infra_links(state: &AppState, domain_id: Option<&str>) -> Result<()> {
    // ip -> [infra_id] (an IP could, in theory, be shared by multiple infra).
    // Join infra so any dangling infra_ip row (e.g. from a delete that bypassed
    // FK cascade) is excluded — otherwise link_infra would 404 on a missing infra.
    let ip_rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT infra_ip.ip, infra_ip.infra_id FROM infra_ip \
         JOIN infra ON infra.id = infra_ip.infra_id",
    )
    .fetch_all(&state.pool)
    .await?;
    if ip_rows.is_empty() {
        return Ok(());
    }
    let mut ip_to_infra: HashMap<String, Vec<String>> = HashMap::new();
    for (ip, infra_id) in ip_rows {
        ip_to_infra.entry(ip).or_default().push(infra_id);
    }

    // Applications linked to a domain.
    let app_domains: Vec<(String, String)> = if let Some(id) = domain_id {
        sqlx::query_as(
            "SELECT ad.application_id, d.fqdn FROM application_domain ad \
             JOIN domain d ON d.id = ad.domain_id WHERE d.id = ?1",
        )
        .bind(id)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT ad.application_id, d.fqdn FROM application_domain ad \
             JOIN domain d ON d.id = ad.domain_id",
        )
        .fetch_all(&state.pool)
        .await?
    };
    for (app_id, fqdn) in app_domains {
        for (ip, infra_id) in resolve_matches(state, &fqdn, &ip_to_infra).await {
            application::link_infra(
                &state.pool,
                &app_id,
                &infra_id,
                Some(&format!("linked through {ip}")),
            )
            .await?;
        }
    }

    // Services targeted by a domain.
    let svc_domains: Vec<(String, String)> = if let Some(id) = domain_id {
        sqlx::query_as(
            "SELECT target_service_id, fqdn FROM domain \
             WHERE target_service_id IS NOT NULL AND id = ?1",
        )
        .bind(id)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT target_service_id, fqdn FROM domain WHERE target_service_id IS NOT NULL",
        )
        .fetch_all(&state.pool)
        .await?
    };
    for (service_id, fqdn) in svc_domains {
        for (ip, infra_id) in resolve_matches(state, &fqdn, &ip_to_infra).await {
            service_svc::link_infra(
                &state.pool,
                &service_id,
                &infra_id,
                Some(&format!("linked through {ip}")),
            )
            .await?;
        }
    }

    Ok(())
}

/// Resolve a fqdn and return `(ip, infra_id)` pairs for each A/AAAA IP that
/// matches a known infra IP. Resolution failures yield no matches (logged).
async fn resolve_matches(
    state: &AppState,
    fqdn: &str,
    ip_to_infra: &HashMap<String, Vec<String>>,
) -> Vec<(String, String)> {
    let lookup = match dns::lookup(state, fqdn).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("reconcile: {fqdn} failed to resolve: {e}");
            return Vec::new();
        }
    };
    let mut out = Vec::new();
    for record in &lookup.records {
        if record.record_type != "A" && record.record_type != "AAAA" {
            continue;
        }
        let ip = normalize_ip(&record.value);
        if let Some(infra_ids) = ip_to_infra.get(&ip) {
            for infra_id in infra_ids {
                out.push((ip.clone(), infra_id.clone()));
            }
        }
    }
    out
}
