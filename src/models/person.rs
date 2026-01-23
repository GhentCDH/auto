use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Person entity - developers, maintainers, support contacts
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
#[derive(Debug, Deserialize)]
pub struct CreatePerson {
    pub name: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
    #[serde(default = "default_active")]
    pub is_active: bool,
    pub notes: Option<String>,
}

/// DTO for updating a person
#[derive(Debug, Deserialize)]
pub struct UpdatePerson {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
}

fn default_active() -> bool {
    true
}

/// Person relation for application detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Serialize)]
pub struct PersonWithRelations {
    #[serde(flatten)]
    pub person: Person,
    pub applications: Vec<ApplicationPersonRelation>,
}

/// Application relation for person detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApplicationPersonRelation {
    pub id: String,
    pub name: String,
    pub status: String,
    pub contribution_type: String,
}
