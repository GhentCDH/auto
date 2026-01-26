mod application;
mod domain;
mod host;
mod network_share;
mod note;
mod person;

pub use application::*;
pub use domain::*;
pub use host::*;
pub use network_share::*;
pub use note::*;
pub use person::*;

use serde::{Deserialize, Serialize};

/// Common pagination parameters
#[derive(Debug, Deserialize, Default)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
}

impl PaginationParams {
    pub fn limit(&self) -> u32 {
        self.per_page.unwrap_or(50).min(100)
    }

    pub fn offset(&self) -> u32 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, params: &PaginationParams) -> Self {
        let per_page = params.limit();
        let page = params.page.unwrap_or(1).max(1);
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

        Self {
            data,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}

/// Generate a new UUID for entity IDs
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
