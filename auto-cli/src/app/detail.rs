use serde_json::Value;

use crate::api::EntityKind;
use crate::view;

use super::Loadable;

pub use crate::view::RelationItem;

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

    pub fn scalar_fields(&self) -> Vec<(String, String)> {
        self.value().map(view::scalar_fields).unwrap_or_default()
    }

    pub fn relation_items(&self) -> Vec<RelationItem> {
        self.value().map(view::relation_items).unwrap_or_default()
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
