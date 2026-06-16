//! On-demand Zabbix proxy for infra **load** (CPU / memory / swap %).
//!
//! Unlike Kuma (push-based, cached in memory), Zabbix is pull-based and already
//! stores history, so we proxy on demand — no poller, no SSE. Infra rows map to
//! Zabbix hosts dynamically by IP (`infra_ip.ip` ↔ host interface IP); no DB
//! column. The endpoints no-op (empty result) when Zabbix isn't configured.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::{Value, json};
use utoipa::ToSchema;

use crate::service::dns;
use crate::{AppState, Config, Error, Result};

// Zabbix item keys. Swap has no util key, so we derive it from pfree.
const CPU_KEY: &str = "system.cpu.util";
const MEM_KEY: &str = "vm.memory.utilization";
const SWAP_PFREE_KEY: &str = "system.swap.size[,pfree]";

/// Current load for one host, as percentages. Fields are `None` when Zabbix has
/// no recent value for that metric.
#[derive(Debug, Clone, Default, Serialize, ToSchema)]
pub struct Load {
    pub cpu: Option<f64>,
    pub mem: Option<f64>,
    pub swap: Option<f64>,
}

/// One timestamped sample. The history endpoint returns a sorted `Vec`; the last
/// element is the current value (no separate "current" struct).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LoadPoint {
    /// Unix epoch seconds.
    pub t: i64,
    pub cpu: Option<f64>,
    pub mem: Option<f64>,
    pub swap: Option<f64>,
}

/// True when both URL and token are set.
pub fn enabled(config: &Config) -> bool {
    config.zabbix_url.is_some() && config.zabbix_token.is_some()
}

/// One JSON-RPC call. Returns the `result` value, or an error on transport
/// failure or a Zabbix `error` object.
async fn call(state: &AppState, method: &str, params: Value) -> Result<Value> {
    let (url, token) = match (&state.config.zabbix_url, &state.config.zabbix_token) {
        (Some(u), Some(t)) => (u, t),
        _ => return Err(Error::InternalError("Zabbix not configured".into())),
    };
    // ponytail: join works for a root-path base; breaks if zabbix_url has a
    // non-root path without a trailing slash. Fine for the configured host.
    let endpoint = url
        .join("api_jsonrpc.php")
        .map_err(|e| Error::InternalError(format!("bad ZABBIX_URL: {e}")))?;

    let body = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "auth": token,
        "id": 1,
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| Error::InternalError(format!("http client: {e}")))?;
    let resp = client
        .post(endpoint)
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::InternalError(format!("Zabbix request failed: {e}")))?;
    let v: Value = resp
        .json()
        .await
        .map_err(|e| Error::InternalError(format!("Zabbix decode failed: {e}")))?;
    if let Some(err) = v.get("error") {
        return Err(Error::InternalError(format!("Zabbix API error: {err}")));
    }
    Ok(v.get("result").cloned().unwrap_or(Value::Null))
}

/// Map every Zabbix host's main interface IP → hostid. `useip=0` hosts have only
/// a DNS name, so resolve it via the shared resolver (a handful of hosts).
async fn ip_to_hostid(state: &AppState) -> Result<HashMap<String, String>> {
    let res = call(
        state,
        "hostinterface.get",
        json!({"output": ["hostid", "ip", "dns", "useip"], "filter": {"main": "1"}}),
    )
    .await?;

    let mut map = HashMap::new();
    for iface in res.as_array().into_iter().flatten() {
        let Some(hostid) = iface.get("hostid").and_then(Value::as_str) else {
            continue;
        };
        let useip = iface.get("useip").and_then(Value::as_str) == Some("1");
        if useip {
            if let Some(ip) = iface.get("ip").and_then(Value::as_str)
                && !ip.is_empty()
            {
                map.insert(ip.to_string(), hostid.to_string());
            }
        } else if let Some(fqdn) = iface.get("dns").and_then(Value::as_str)
            && !fqdn.is_empty()
            && let Ok(lookup) = dns::lookup(state, fqdn).await
        {
            for rec in lookup.records {
                if rec.record_type == "A" || rec.record_type == "AAAA" {
                    map.insert(rec.value, hostid.to_string());
                }
            }
        }
    }
    Ok(map)
}

/// infra_id → hostid for every infra whose stored IP matches a Zabbix host.
async fn infra_to_hostid(state: &AppState) -> Result<HashMap<String, String>> {
    let ipmap = ip_to_hostid(state).await?;
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT infra_id, ip FROM infra_ip")
        .fetch_all(&state.pool)
        .await?;
    let mut out = HashMap::new();
    for (infra_id, ip) in rows {
        if let Some(hostid) = ipmap.get(&ip) {
            out.entry(infra_id).or_insert_with(|| hostid.clone());
        }
    }
    Ok(out)
}

fn parse_val(item: &Value) -> Option<f64> {
    item.get("lastvalue")
        .and_then(Value::as_str)
        .and_then(|s| s.parse().ok())
}

/// Apply one item's value to the right field of a `Load`. Swap is derived from
/// pfree (100 − free%).
fn apply_metric(load: &mut Load, key: &str, val: Option<f64>) {
    match key {
        CPU_KEY => load.cpu = val,
        MEM_KEY => load.mem = val,
        SWAP_PFREE_KEY => load.swap = val.map(|v| 100.0 - v),
        _ => {}
    }
}

/// Current load for all infra that match a Zabbix host. One bulk `item.get`.
pub async fn loads_for_infra(state: &AppState) -> Result<HashMap<String, Load>> {
    if !enabled(&state.config) {
        return Ok(HashMap::new());
    }
    let infra_host = infra_to_hostid(state).await?;
    if infra_host.is_empty() {
        return Ok(HashMap::new());
    }
    let hostids: Vec<&String> = infra_host
        .values()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let items = call(
        state,
        "item.get",
        json!({
            "output": ["hostid", "key_", "lastvalue"],
            "hostids": hostids,
            "filter": {"key_": [CPU_KEY, MEM_KEY, SWAP_PFREE_KEY]},
        }),
    )
    .await?;

    let mut host_load: HashMap<String, Load> = HashMap::new();
    for item in items.as_array().into_iter().flatten() {
        let (Some(hostid), Some(key)) = (
            item.get("hostid").and_then(Value::as_str),
            item.get("key_").and_then(Value::as_str),
        ) else {
            continue;
        };
        apply_metric(
            host_load.entry(hostid.to_string()).or_default(),
            key,
            parse_val(item),
        );
    }

    Ok(infra_host
        .into_iter()
        .filter_map(|(infra_id, hostid)| host_load.get(&hostid).map(|l| (infra_id, l.clone())))
        .collect())
}

/// Recent history for one infra. Empty when Zabbix is off or the infra has no
/// matching host. Buckets the three metrics by sample clock onto one time axis.
pub async fn history_for_infra(
    state: &AppState,
    infra_id: &str,
    hours: i64,
) -> Result<Vec<LoadPoint>> {
    if !enabled(&state.config) {
        return Ok(Vec::new());
    }
    let infra_host = infra_to_hostid(state).await?;
    let Some(hostid) = infra_host.get(infra_id) else {
        return Ok(Vec::new());
    };

    // itemid → metric key, for this host's three metrics.
    let items = call(
        state,
        "item.get",
        json!({
            "output": ["itemid", "key_"],
            "hostids": hostid,
            "filter": {"key_": [CPU_KEY, MEM_KEY, SWAP_PFREE_KEY]},
        }),
    )
    .await?;
    let mut item_key: HashMap<String, String> = HashMap::new();
    for item in items.as_array().into_iter().flatten() {
        if let (Some(id), Some(key)) = (
            item.get("itemid").and_then(Value::as_str),
            item.get("key_").and_then(Value::as_str),
        ) {
            item_key.insert(id.to_string(), key.to_string());
        }
    }
    if item_key.is_empty() {
        return Ok(Vec::new());
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let from = now - hours * 3600;
    let itemids: Vec<&String> = item_key.keys().collect();

    let hist = call(
        state,
        "history.get",
        json!({
            "output": ["itemid", "clock", "value"],
            "itemids": itemids,
            "history": 0, // all three metrics are float
            "time_from": from,
            "sortfield": "clock",
            "sortorder": "ASC",
        }),
    )
    .await?;

    let mut points: BTreeMap<i64, LoadPoint> = BTreeMap::new();
    for h in hist.as_array().into_iter().flatten() {
        let (Some(itemid), Some(clock), Some(val)) = (
            h.get("itemid").and_then(Value::as_str),
            h.get("clock")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<i64>().ok()),
            h.get("value")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<f64>().ok()),
        ) else {
            continue;
        };
        let Some(key) = item_key.get(itemid) else {
            continue;
        };
        let p = points.entry(clock).or_insert(LoadPoint {
            t: clock,
            cpu: None,
            mem: None,
            swap: None,
        });
        // Reuse apply_metric via a scratch Load, then copy across.
        let mut scratch = Load {
            cpu: p.cpu,
            mem: p.mem,
            swap: p.swap,
        };
        apply_metric(&mut scratch, key, Some(val));
        p.cpu = scratch.cpu;
        p.mem = scratch.mem;
        p.swap = scratch.swap;
    }

    Ok(points.into_values().collect())
}
