use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Person entity - developers, maintainers, support contacts
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new person
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePerson {
    pub name: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
}

impl CreatePerson {
    /// Fill omitted defaultable fields from configured defaults.
    pub fn apply_defaults(&mut self, d: &crate::config::PersonDefaults) {
        self.is_active.get_or_insert(d.is_active);
    }
}

/// DTO for updating a person
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePerson {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
}

/// Person relation for application detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PersonRelation {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub is_active: bool,
    pub contribution_type: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub relation_notes: Option<String>,
}

/// DTO for linking a person to an application
#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkPerson {
    #[serde(default = "default_contribution")]
    pub contribution_type: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
}

fn default_contribution() -> String {
    "developer".to_string()
}

/// Person with related applications
#[derive(Debug, Serialize, ToSchema)]
pub struct PersonWithRelations {
    #[serde(flatten)]
    pub person: Person,
    pub applications: Vec<ApplicationPersonRelation>,
}

/// Application relation for person detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ApplicationPersonRelation {
    pub id: String,
    pub name: String,
    pub status: String,
    pub contribution_type: String,
}
