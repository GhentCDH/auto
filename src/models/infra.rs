use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Infra entity - infrastructure like nomad clusters, servers, k8s clusters, etc.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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

/// An IP address associated with infra — either resolved from its domain
/// (`source = "domain"`) or assigned manually (`source = "manual"`).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct InfraIp {
    pub ip: String,
    pub source: String,
    pub last_synced_at: String,
}

/// The domain that targets an infra (if any), for prefilling the editor.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct InfraDomainRef {
    pub id: String,
    pub fqdn: String,
}

/// A brand-new domain to create alongside an infra. Has no target field: the
/// server points it at the infra being created (`domain.target_infra_id`),
/// which is the only way to resolve the otherwise-circular create order
/// (a domain requires a target, an infra resolves IPs from its domain).
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewInfraDomain {
    pub fqdn: String,
    pub registrar: Option<String>,
    pub dns_provider: Option<String>,
    pub expires_at: Option<String>,
    pub notes: Option<String>,
}

/// DTO for creating a new infra.
///
/// `domain_id` (optional) sets an existing domain's target to this infra, so its
/// IPs are resolved from DNS. `new_domain` (optional) creates a fresh domain
/// targeting this infra in the same request. `manual_ips` (optional) assigns
/// fixed IPs directly.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInfra {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub infra_type: String,
    pub domain_id: Option<String>,
    pub new_domain: Option<NewInfraDomain>,
    pub manual_ips: Option<Vec<String>>,
}

/// DTO for updating an infra. See [`CreateInfra`] for the field semantics.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateInfra {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub infra_type: Option<String>,
    pub domain_id: Option<String>,
    pub new_domain: Option<NewInfraDomain>,
    pub manual_ips: Option<Vec<String>>,
}

/// Infra relation for embedding in Application/Service detail views
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct InfraRelation {
    pub id: String,
    pub name: String,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub infra_type: String,
    pub relation_notes: Option<String>,
}

/// DTO for linking infra to an application or service
#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkInfra {
    pub notes: Option<String>,
}

/// Infra with related entities
#[derive(Debug, Serialize, ToSchema)]
pub struct InfraWithRelations {
    #[serde(flatten)]
    pub infra: Infra,
    pub ips: Vec<InfraIp>,
    /// The domain targeting this infra, if one does (source of domain IPs).
    pub domain: Option<InfraDomainRef>,
    pub applications: Vec<ApplicationInfraRelation>,
    pub services: Vec<ServiceInfraRelation>,
}

/// Application relation for infra detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ApplicationInfraRelation {
    pub id: String,
    pub name: String,
    pub environment: String,
    pub status: String,
}

/// Service relation for infra detail view
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ServiceInfraRelation {
    pub id: String,
    pub name: String,
    pub environment: String,
    pub status: String,
}
