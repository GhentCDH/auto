use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Client entity - who applications were built for
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new client
#[derive(Debug, Deserialize)]
pub struct CreateClient {
    pub name: String,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    pub notes: Option<String>,
}

/// DTO for updating a client
#[derive(Debug, Deserialize)]
pub struct UpdateClient {
    pub name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub department: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

fn default_status() -> String {
    "active".to_string()
}

/// Client relation for application detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClientRelation {
    pub id: String,
    pub name: String,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub status: String,
    pub relationship_type: String,
    pub contract_ref: Option<String>,
    pub relation_notes: Option<String>,
}

/// DTO for linking a client to an application
#[derive(Debug, Deserialize)]
pub struct LinkClient {
    #[serde(default = "default_relationship")]
    pub relationship_type: String,
    pub contract_ref: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
}

fn default_relationship() -> String {
    "customer".to_string()
}

/// Client with related applications
#[derive(Debug, Serialize)]
pub struct ClientWithRelations {
    #[serde(flatten)]
    pub client: Client,
    pub applications: Vec<ApplicationClientRelation>,
}

/// Application relation for client detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApplicationClientRelation {
    pub id: String,
    pub name: String,
    pub status: String,
    pub relationship_type: String,
}
