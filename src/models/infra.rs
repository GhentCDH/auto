use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Infra entity - infrastructure like nomad clusters, servers, k8s clusters, etc.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Infra {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub infra_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new infra
#[derive(Debug, Deserialize)]
pub struct CreateInfra {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub infra_type: String,
}

/// DTO for updating an infra
#[derive(Debug, Deserialize)]
pub struct UpdateInfra {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub infra_type: Option<String>,
}

/// Infra relation for embedding in Application/Service detail views
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InfraRelation {
    pub id: String,
    pub name: String,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub infra_type: String,
    pub relation_notes: Option<String>,
}

/// DTO for linking infra to an application or service
#[derive(Debug, Deserialize)]
pub struct LinkInfra {
    pub notes: Option<String>,
}

/// Infra with related entities
#[derive(Debug, Serialize)]
pub struct InfraWithRelations {
    #[serde(flatten)]
    pub infra: Infra,
    pub applications: Vec<ApplicationInfraRelation>,
    pub services: Vec<ServiceInfraRelation>,
}

/// Application relation for infra detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApplicationInfraRelation {
    pub id: String,
    pub name: String,
    pub environment: String,
    pub status: String,
}

/// Service relation for infra detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServiceInfraRelation {
    pub id: String,
    pub name: String,
    pub environment: String,
    pub status: String,
}
