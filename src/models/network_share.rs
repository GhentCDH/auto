use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// NetworkShare entity - SMB/NFS shares
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NetworkShare {
    pub id: String,
    pub name: String,
    pub path: String,
    pub share_type: String,
    pub server: Option<String>,
    pub purpose: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new network share
#[derive(Debug, Deserialize)]
pub struct CreateNetworkShare {
    pub name: String,
    pub path: String,
    #[serde(default = "default_share_type")]
    pub share_type: String,
    pub server: Option<String>,
    pub purpose: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    pub notes: Option<String>,
}

/// DTO for updating a network share
#[derive(Debug, Deserialize)]
pub struct UpdateNetworkShare {
    pub name: Option<String>,
    pub path: Option<String>,
    pub share_type: Option<String>,
    pub server: Option<String>,
    pub purpose: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

fn default_share_type() -> String {
    "smb".to_string()
}

fn default_status() -> String {
    "active".to_string()
}

/// NetworkShare relation for application detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NetworkShareRelation {
    pub id: String,
    pub name: String,
    pub path: String,
    pub share_type: String,
    pub server: Option<String>,
    pub status: String,
    pub usage: Option<String>,
    pub mount_point: Option<String>,
    pub permissions: Option<String>,
    pub relation_notes: Option<String>,
}

/// DTO for linking a network share to an application
#[derive(Debug, Deserialize)]
pub struct LinkNetworkShare {
    pub usage: Option<String>,
    pub mount_point: Option<String>,
    pub permissions: Option<String>,
    pub notes: Option<String>,
}

/// NetworkShare with related applications
#[derive(Debug, Serialize)]
pub struct NetworkShareWithRelations {
    #[serde(flatten)]
    pub network_share: NetworkShare,
    pub applications: Vec<ApplicationNetworkShareRelation>,
}

/// Application relation for network share detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApplicationNetworkShareRelation {
    pub id: String,
    pub name: String,
    pub status: String,
    pub usage: Option<String>,
    pub mount_point: Option<String>,
}
