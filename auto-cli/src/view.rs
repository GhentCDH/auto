//! Presentation helpers shared by the TUI and the CLI output: which columns a
//! list shows, how a JSON value becomes a cell, and how a detail body splits
//! into scalar fields and relation sections.

use serde_json::Value;

use crate::api::EntityKind;

/// A related entity inside a detail body.
#[derive(Debug, Clone)]
pub struct RelationItem {
    pub section: String,
    pub kind: Option<EntityKind>,
    pub id: Option<String>,
    pub label: String,
}

/// Per-kind list columns: (header, JSON key) pairs rendered in order.
pub fn columns(kind: EntityKind) -> &'static [(&'static str, &'static str)] {
    match kind {
        EntityKind::Applications => &[
            ("Name", "name"),
            ("Env", "environment"),
            ("Status", "status"),
            ("URL", "url"),
        ],
        EntityKind::Services => &[
            ("Name", "name"),
            ("Env", "environment"),
            ("Status", "status"),
            ("Description", "description"),
        ],
        EntityKind::Infra => &[
            ("Name", "name"),
            ("Type", "infra_type"),
            ("Description", "description"),
        ],
        EntityKind::Domains => &[
            ("FQDN", "fqdn"),
            ("Registrar", "registrar"),
            ("DNS provider", "dns_provider"),
            ("Expires", "expires_at"),
        ],
        EntityKind::People => &[
            ("Name", "name"),
            ("Email", "email"),
            ("Role", "role"),
            ("Active", "is_active"),
        ],
        EntityKind::Shares => &[
            ("Name", "name"),
            ("Type", "share_type"),
            ("Server", "server"),
            ("Status", "status"),
        ],
        EntityKind::Stacks => &[("Name", "name"), ("Notes", "notes")],
        EntityKind::Healthchecks => &[
            ("Name", "name"),
            ("Domain", "domain_fqdn"),
            ("Proto", "protocol"),
            ("Enabled", "is_enabled"),
            ("Dirty", "kuma_dirty"),
        ],
        EntityKind::Notes => &[
            ("Title", "title"),
            ("Type", "note_type"),
            ("On", "entity_type"),
            ("Pinned", "is_pinned"),
        ],
    }
}

/// Human label for a row: `name` for most kinds, `fqdn` for domains, `title`
/// for notes.
pub fn row_label(row: &Value) -> &str {
    ["name", "fqdn", "title"]
        .iter()
        .find_map(|key| row.get(key).and_then(Value::as_str))
        .unwrap_or("?")
}

/// Render any scalar JSON value as a short cell string.
pub fn cell_text(row: &Value, key: &str) -> String {
    match row.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(true)) => "yes".into(),
        Some(Value::Bool(false)) => "no".into(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// Scalar fields in display order: id and name first, then the rest as they
/// appear, empty values skipped.
pub fn scalar_fields(value: &Value) -> Vec<(String, String)> {
    const FIRST: [&str; 3] = ["id", "name", "fqdn"];
    let Value::Object(map) = value else {
        return Vec::new();
    };
    let first = FIRST
        .iter()
        .filter_map(|key| scalar_text(map.get(*key)).map(|text| (key.to_string(), text)));
    let rest = map
        .iter()
        .filter(|(key, _)| !FIRST.contains(&key.as_str()))
        .filter_map(|(key, value)| scalar_text(Some(value)).map(|text| (key.clone(), text)));
    first.chain(rest).collect()
}

/// All relation entries flattened across array-valued keys, in key order.
pub fn relation_items(value: &Value) -> Vec<RelationItem> {
    let Value::Object(map) = value else {
        return Vec::new();
    };
    map.iter()
        .filter_map(|(key, value)| value.as_array().map(|entries| (key, entries)))
        .flat_map(|(key, entries)| {
            let kind = section_kind(key);
            entries.iter().map(move |entry| RelationItem {
                section: key.clone(),
                kind,
                id: entry.get("id").and_then(Value::as_str).map(String::from),
                label: relation_label(entry),
            })
        })
        .collect()
}

fn scalar_text(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(if *b { "yes".into() } else { "no".into() }),
        _ => None,
    }
}

/// Map a relation section key to the entity kind it drills into.
/// `None` for sections the TUI does not drill into (e.g. notes, ips).
pub fn section_kind(section: &str) -> Option<EntityKind> {
    Some(match section {
        "applications" => EntityKind::Applications,
        "services" => EntityKind::Services,
        "infra" => EntityKind::Infra,
        "domains" => EntityKind::Domains,
        "people" => EntityKind::People,
        "network_shares" => EntityKind::Shares,
        "stacks" => EntityKind::Stacks,
        "healthchecks" => EntityKind::Healthchecks,
        _ => return None,
    })
}

fn relation_label(entry: &Value) -> String {
    ["name", "fqdn", "title", "ip", "content"]
        .iter()
        .find_map(|key| {
            entry
                .get(key)
                .and_then(Value::as_str)
                .filter(|text| !text.is_empty())
        })
        .or_else(|| entry.get("id").and_then(Value::as_str))
        .unwrap_or("?")
        .to_string()
}
