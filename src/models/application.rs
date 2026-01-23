use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Application entity - the central entity in the system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new application
#[derive(Debug, Deserialize)]
pub struct CreateApplication {
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
}

/// DTO for updating an application
#[derive(Debug, Deserialize)]
pub struct UpdateApplication {
    pub name: Option<String>,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub status: Option<String>,
}

fn default_status() -> String {
    "active".to_string()
}

/// Application with all related entities
#[derive(Debug, Serialize)]
pub struct ApplicationWithRelations {
    #[serde(flatten)]
    pub application: Application,
    pub hosts: Vec<super::HostRelation>,
    pub domains: Vec<super::DomainRelation>,
    pub people: Vec<super::PersonRelation>,
    pub clients: Vec<super::ClientRelation>,
    pub network_shares: Vec<super::NetworkShareRelation>,
    pub notes: Vec<super::Note>,
}
