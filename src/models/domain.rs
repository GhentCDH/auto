use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Domain entity - DNS records
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Domain {
    pub id: String,
    pub fqdn: String,
    pub registrar: Option<String>,
    pub dns_provider: Option<String>,
    pub expires_at: Option<String>,
    pub notes: Option<String>,
    pub target_application_id: Option<String>,
    pub target_service_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new domain
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDomain {
    pub fqdn: String,
    pub registrar: Option<String>,
    pub dns_provider: Option<String>,
    pub expires_at: Option<String>,
    pub notes: Option<String>,
    pub target_application_id: Option<String>,
    pub target_service_id: Option<String>,
}

/// DTO for updating a domain
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDomain {
    pub fqdn: Option<String>,
    pub registrar: Option<String>,
    pub dns_provider: Option<String>,
    pub expires_at: Option<String>,
    pub notes: Option<String>,
    pub target_application_id: Option<String>,
    pub target_service_id: Option<String>,
}

/// Domain relation for application detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DomainRelation {
    pub id: String,
    pub fqdn: String,
    pub target_application_id: Option<String>,
    pub target_application_name: Option<String>,
    pub target_service_id: Option<String>,
    pub target_service_name: Option<String>,
    pub relation_notes: Option<String>,
}

/// DTO for linking a domain to an application
#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkDomain {
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub enum DomainTarget {
    #[serde(rename = "application")]
    Application { id: String, name: String },
    #[serde(rename = "service")]
    Service { id: String, name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TargetName {
    pub name: String,
}

/// Domain with related applications
#[derive(Debug, Serialize, ToSchema)]
pub struct DomainWithRelations {
    #[serde(flatten)]
    pub domain: Domain,
    pub target_application_name: Option<String>,
    pub target_service_name: Option<String>,
    pub applications: Vec<ApplicationDomainRelation>,
}

/// Application relation for domain detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ApplicationDomainRelation {
    pub id: String,
    pub name: String,
}
