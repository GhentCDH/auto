use serde_json::Value;

use crate::api::EntityKind;

use super::Loadable;

/// One level of the detail drill-down stack.
#[derive(Debug)]
pub struct DetailView {
    pub kind: EntityKind,
    pub id: String,
    /// Fresh fetch result; seeded `Ready` from the list row (lists already
    /// embed relations) so the view renders instantly while reloading.
    pub data: Loadable<Value>,
    /// Index into `relation_items` (flattened across sections).
    pub selected: usize,
}

/// A selectable related entity inside a detail view.
#[derive(Debug, Clone)]
pub struct RelationItem {
    pub section: String,
    pub kind: Option<EntityKind>,
    pub id: Option<String>,
    pub label: String,
}

impl DetailView {
    pub fn new(kind: EntityKind, id: String, seed: Option<Value>) -> Self {
        Self {
            kind,
            id,
            data: seed.map(Loadable::Ready).unwrap_or(Loadable::Loading),
            selected: 0,
        }
    }

    pub fn value(&self) -> Option<&Value> {
        match &self.data {
            Loadable::Ready(value) => Some(value),
            _ => None,
        }
    }

    /// Scalar fields in display order: id and name first, then the rest as
    /// they appear, empty values skipped.
    pub fn scalar_fields(&self) -> Vec<(String, String)> {
        let Some(Value::Object(map)) = self.value() else {
            return Vec::new();
        };
        let mut fields: Vec<(String, String)> = Vec::new();
        for key in ["id", "name", "fqdn"] {
            if let Some(text) = scalar_text(map.get(key)) {
                fields.push((key.to_string(), text));
            }
        }
        for (key, value) in map {
            if ["id", "name", "fqdn"].contains(&key.as_str()) {
                continue;
            }
            if let Some(text) = scalar_text(Some(value)) {
                fields.push((key.clone(), text));
            }
        }
        fields
    }

    /// All relation entries flattened across array-valued keys, in key order.
    pub fn relation_items(&self) -> Vec<RelationItem> {
        let Some(Value::Object(map)) = self.value() else {
            return Vec::new();
        };
        let mut items = Vec::new();
        for (key, value) in map {
            let Value::Array(entries) = value else {
                continue;
            };
            let kind = section_kind(key);
            for entry in entries {
                items.push(RelationItem {
                    section: key.clone(),
                    kind,
                    id: entry.get("id").and_then(Value::as_str).map(String::from),
                    label: relation_label(entry),
                });
            }
        }
        items
    }

    pub fn selected_item(&self) -> Option<RelationItem> {
        self.relation_items().into_iter().nth(self.selected)
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.relation_items().len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        self.selected = (self.selected as isize + delta).clamp(0, len as isize - 1) as usize;
    }
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
/// `None` for non-entity sections (e.g. notes, ips).
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
    for key in ["name", "fqdn", "title", "ip", "content"] {
        if let Some(text) = entry.get(key).and_then(Value::as_str)
            && !text.is_empty()
        {
            return text.to_string();
        }
    }
    entry
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_string()
}
