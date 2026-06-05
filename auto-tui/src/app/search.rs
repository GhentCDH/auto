use crate::api::EntityKind;
use crate::api::models::{SearchResult, SearchResults};

use super::Loadable;

/// Global search overlay state (opened with `/`).
#[derive(Debug, Default)]
pub struct SearchState {
    pub open: bool,
    pub input: String,
    pub results: Loadable<SearchResults>,
    pub selected: usize,
}

impl SearchState {
    /// Results flattened across entity groups, paired with their kind.
    pub fn flat(&self) -> Vec<(EntityKind, &SearchResult)> {
        let Loadable::Ready(results) = &self.results else {
            return Vec::new();
        };
        let groups: [(EntityKind, &Vec<SearchResult>); 8] = [
            (EntityKind::Applications, &results.applications),
            (EntityKind::Services, &results.services),
            (EntityKind::Infra, &results.infra),
            (EntityKind::Domains, &results.domains),
            (EntityKind::People, &results.people),
            (EntityKind::Shares, &results.network_shares),
            (EntityKind::Stacks, &results.stacks),
            (EntityKind::Healthchecks, &results.healthchecks),
        ];
        groups
            .into_iter()
            .flat_map(|(kind, items)| items.iter().map(move |item| (kind, item)))
            .collect()
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.flat().len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        self.selected = (self.selected as isize + delta).clamp(0, len as isize - 1) as usize;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.input.clear();
        self.results = Loadable::Idle;
        self.selected = 0;
    }
}
