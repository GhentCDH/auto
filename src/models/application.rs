use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Application entity - the central entity in the system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub environment: String,
    pub url: Option<String>,
    pub status: String,
    pub image_refs: Option<String>,
    pub outline_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new application
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApplication {
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub environment: Option<String>,
    pub url: Option<String>,
    pub status: Option<String>,
    pub image_refs: Option<String>,
    pub outline_url: Option<String>,
}

impl CreateApplication {
    /// Fill omitted defaultable fields from configured defaults.
    pub fn apply_defaults(&mut self, d: &crate::config::ApplicationDefaults) {
        self.environment
            .get_or_insert_with(|| d.environment.clone());
        self.status.get_or_insert_with(|| d.status.clone());
    }
}

/// DTO for updating an application
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateApplication {
    pub name: Option<String>,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub environment: Option<String>,
    pub url: Option<String>,
    pub status: Option<String>,
    pub image_refs: Option<String>,
    pub outline_url: Option<String>,
}

/// Application with all related entities
#[derive(Debug, Serialize, ToSchema)]
pub struct ApplicationWithRelations {
    #[serde(flatten)]
    pub application: Application,
    pub infra: Vec<super::InfraRelation>,
    pub services: Vec<super::ServiceRelation>,
    pub domains: Vec<super::DomainRelation>,
    pub people: Vec<super::PersonRelation>,
    pub network_shares: Vec<super::NetworkShareRelation>,
    pub notes: Vec<super::Note>,
    pub stacks: Vec<super::StackRelation>,
    pub healthchecks: Vec<super::HealthcheckRelation>,
}
