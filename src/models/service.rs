use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Service entity - shared services like elasticsearch, load balancers, etc.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub environment: String,
    pub status: String,
    pub image_refs: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new service
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateService {
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(default = "default_status")]
    pub status: String,
    pub image_refs: Option<String>,
}

/// DTO for updating a service
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateService {
    pub name: Option<String>,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub environment: Option<String>,
    pub status: Option<String>,
    pub image_refs: Option<String>,
}

fn default_environment() -> String {
    "prd".to_string()
}

fn default_status() -> String {
    "active".to_string()
}

/// Service relation for embedding in Application/Infra detail views
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ServiceRelation {
    pub id: String,
    pub name: String,
    pub environment: String,
    pub status: String,
    pub relation_notes: Option<String>,
}

/// DTO for linking a service to an application
#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkService {
    pub notes: Option<String>,
}

/// Service with related entities
#[derive(Debug, Serialize, ToSchema)]
pub struct ServiceWithRelations {
    #[serde(flatten)]
    pub service: Service,
    pub applications: Vec<ApplicationServiceRelation>,
    pub infra: Vec<super::InfraRelation>,
    pub healthchecks: Vec<super::HealthcheckRelation>,
}

/// Application relation for service detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ApplicationServiceRelation {
    pub id: String,
    pub name: String,
    pub environment: String,
    pub status: String,
}
