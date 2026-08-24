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
    pub outline_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new service
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateService {
    #[serde(deserialize_with = "super::trim_str")]
    pub name: String,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub repository_url: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub environment: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub status: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub image_refs: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub outline_url: Option<String>,
}

impl CreateService {
    /// Fill omitted defaultable fields from configured defaults.
    pub fn apply_defaults(&mut self, d: &crate::config::ServiceDefaults) {
        self.environment
            .get_or_insert_with(|| d.environment.clone());
        self.status.get_or_insert_with(|| d.status.clone());
    }
}

/// DTO for updating a service
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateService {
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub repository_url: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub environment: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub status: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub image_refs: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub outline_url: Option<String>,
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
    #[serde(default, deserialize_with = "super::trim_opt_str")]
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
