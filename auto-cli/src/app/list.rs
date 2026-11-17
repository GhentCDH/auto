use serde_json::Value;

use crate::api::models::PaginatedResponse;

use super::Loadable;

pub use crate::view::{cell_text, columns, row_label};

pub const PER_PAGE: u32 = 50;

/// Browsing state for one entity tab.
#[derive(Debug, Default)]
pub struct EntityList {
    pub data: Loadable<PaginatedResponse<Value>>,
    pub selected: usize,
    pub page: u32,
    /// Applied search filter (already part of the loaded data).
    pub filter: String,
    /// In-progress filter text while the input line is focused.
    pub filter_input: Option<String>,
}

impl EntityList {
    pub fn rows(&self) -> &[Value] {
        match &self.data {
            Loadable::Ready(resp) => &resp.data,
            _ => &[],
        }
    }

    pub fn response(&self) -> Option<&PaginatedResponse<Value>> {
        match &self.data {
            Loadable::Ready(resp) => Some(resp),
            _ => None,
        }
    }

    pub fn selected_row(&self) -> Option<&Value> {
        self.rows().get(self.selected)
    }

    pub fn selected_id(&self) -> Option<String> {
        self.selected_row()?
            .get("id")
            .and_then(Value::as_str)
            .map(String::from)
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.rows().len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        self.selected = (self.selected as isize + delta).clamp(0, len as isize - 1) as usize;
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    pub fn select_last(&mut self) {
        self.selected = self.rows().len().saturating_sub(1);
    }

    /// Clamp the page move against the loaded total; returns true when a
    /// reload is needed.
    pub fn change_page(&mut self, delta: i64) -> bool {
        let Some(resp) = self.response() else {
            return false;
        };
        let target = (self.page as i64 + delta).clamp(1, resp.total_pages.max(1) as i64) as u32;
        if target == self.page {
            return false;
        }
        self.page = target;
        true
    }
}
