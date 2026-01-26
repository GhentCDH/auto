use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Domain entity - DNS records and SSL info
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Domain {
    pub id: String,
    pub name: String,
    pub registrar: Option<String>,
    pub dns_provider: Option<String>,
    pub expires_at: Option<String>,
    pub ssl_expires_at: Option<String>,
    pub ssl_issuer: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new domain
#[derive(Debug, Deserialize)]
pub struct CreateDomain {
    pub name: String,
    pub registrar: Option<String>,
    pub dns_provider: Option<String>,
    pub expires_at: Option<String>,
    pub ssl_expires_at: Option<String>,
    pub ssl_issuer: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    pub notes: Option<String>,
}

/// DTO for updating a domain
#[derive(Debug, Deserialize)]
pub struct UpdateDomain {
    pub name: Option<String>,
    pub registrar: Option<String>,
    pub dns_provider: Option<String>,
    pub expires_at: Option<String>,
    pub ssl_expires_at: Option<String>,
    pub ssl_issuer: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

fn default_status() -> String {
    "active".to_string()
}

/// Domain relation for application detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DomainRelation {
    pub id: String,
    pub name: String,
    pub registrar: Option<String>,
    pub expires_at: Option<String>,
    pub ssl_expires_at: Option<String>,
    pub status: String,
    pub record_type: String,
    pub target: Option<String>,
    pub is_primary: bool,
    pub relation_notes: Option<String>,
}

/// DTO for linking a domain to an application
#[derive(Debug, Deserialize)]
pub struct LinkDomain {
    #[serde(default = "default_record_type")]
    pub record_type: String,
    pub target: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
    pub notes: Option<String>,
}

fn default_record_type() -> String {
    "A".to_string()
}

/// Domain with related applications
#[derive(Debug, Serialize)]
pub struct DomainWithRelations {
    #[serde(flatten)]
    pub domain: Domain,
    pub applications: Vec<ApplicationDomainRelation>,
}

/// Application relation for domain detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApplicationDomainRelation {
    pub id: String,
    pub name: String,
    pub status: String,
    pub record_type: String,
    pub is_primary: bool,
}
