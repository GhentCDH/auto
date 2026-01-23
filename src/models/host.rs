use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Host entity - servers, VMs, Nomad jobs, containers
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Host {
    pub id: String,
    pub name: String,
    pub host_type: String,
    pub hostname: Option<String>,
    pub ip_address: Option<String>,
    pub location: Option<String>,
    pub os: Option<String>,
    pub specs: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new host
#[derive(Debug, Deserialize)]
pub struct CreateHost {
    pub name: String,
    pub host_type: String,
    pub hostname: Option<String>,
    pub ip_address: Option<String>,
    pub location: Option<String>,
    pub os: Option<String>,
    pub specs: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    pub notes: Option<String>,
}

/// DTO for updating a host
#[derive(Debug, Deserialize)]
pub struct UpdateHost {
    pub name: Option<String>,
    pub host_type: Option<String>,
    pub hostname: Option<String>,
    pub ip_address: Option<String>,
    pub location: Option<String>,
    pub os: Option<String>,
    pub specs: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

fn default_status() -> String {
    "active".to_string()
}

/// Host relation for application detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HostRelation {
    pub id: String,
    pub name: String,
    pub host_type: String,
    pub hostname: Option<String>,
    pub ip_address: Option<String>,
    pub status: String,
    pub role: String,
    pub relation_notes: Option<String>,
}

/// DTO for linking a host to an application
#[derive(Debug, Deserialize)]
pub struct LinkHost {
    #[serde(default = "default_role")]
    pub role: String,
    pub notes: Option<String>,
}

fn default_role() -> String {
    "primary".to_string()
}

/// Host with related applications
#[derive(Debug, Serialize)]
pub struct HostWithRelations {
    #[serde(flatten)]
    pub host: Host,
    pub applications: Vec<ApplicationHostRelation>,
}

/// Application relation for host detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApplicationHostRelation {
    pub id: String,
    pub name: String,
    pub status: String,
    pub role: String,
}
