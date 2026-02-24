use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Auto API",
        version = "1.0.1",
        description = "Digital assets management system API\n\n**Authentication:** This API is protected by HTTP Basic Authentication at the reverse proxy level (Caddy). All endpoints require authentication credentials which are validated by the reverse proxy before requests reach this application.",
    ),
    paths(
        // Health endpoints
        crate::api::healthcheck,
        crate::api::version,
        
        // Applications
        crate::api::applications::list,
        crate::api::applications::get_one,
        crate::api::applications::create,
        crate::api::applications::update,
        crate::api::applications::delete_one,
        crate::api::applications::link_infra,
        crate::api::applications::unlink_infra,
        crate::api::applications::link_service,
        crate::api::applications::unlink_service,
        crate::api::applications::link_domain,
        crate::api::applications::unlink_domain,
        crate::api::applications::link_person,
        crate::api::applications::unlink_person,
        crate::api::applications::link_share,
        crate::api::applications::unlink_share,
        crate::api::applications::link_stack,
        crate::api::applications::unlink_stack,
        crate::api::applications::sync_outline,

        // Services
        crate::api::services::list,
        crate::api::services::get_one,
        crate::api::services::create,
        crate::api::services::update,
        crate::api::services::delete_one,
        crate::api::services::link_infra,
        crate::api::services::unlink_infra,
        crate::api::services::sync_outline,

        // Infrastructure
        crate::api::infra::list,
        crate::api::infra::get_one,
        crate::api::infra::create,
        crate::api::infra::update,
        crate::api::infra::delete_one,
        
        // Domains
        crate::api::domains::list,
        crate::api::domains::get_one,
        crate::api::domains::create,
        crate::api::domains::update,
        crate::api::domains::delete_one,
        
        // People
        crate::api::people::list,
        crate::api::people::get_one,
        crate::api::people::create,
        crate::api::people::update,
        crate::api::people::delete_one,
        
        // Network shares
        crate::api::shares::list,
        crate::api::shares::get_one,
        crate::api::shares::create,
        crate::api::shares::update,
        crate::api::shares::delete_one,
        
        // Notes
        crate::api::notes::list,
        crate::api::notes::get_one,
        crate::api::notes::create,
        crate::api::notes::update,
        crate::api::notes::delete_one,
        
        // Stacks
        crate::api::stacks::list,
        crate::api::stacks::get_one,
        crate::api::stacks::create,
        crate::api::stacks::update,
        crate::api::stacks::delete_one,
        
        // Healthchecks
        crate::api::healthchecks::list,
        crate::api::healthchecks::get_one,
        crate::api::healthchecks::create,
        crate::api::healthchecks::update,
        crate::api::healthchecks::delete_one,
        crate::api::healthchecks::execute,
        crate::api::healthchecks::export_kuma,
        crate::api::healthchecks::kuma_endpoint,
        crate::api::healthchecks::sync_kuma_one,
        crate::api::healthchecks::sync_kuma_all,
        
        // Dashboard
        crate::api::dashboard::stats,
        
        // Search
        crate::api::search::global_search,
    ),
    components(
        schemas(
            // Common models
            crate::models::PaginationParams,
            crate::models::PaginatedResponse<crate::models::Application>,
            crate::models::PaginatedResponse<crate::models::Service>,
            crate::models::PaginatedResponse<crate::models::Infra>,
            crate::models::PaginatedResponse<crate::models::Domain>,
            crate::models::PaginatedResponse<crate::models::Person>,
            crate::models::PaginatedResponse<crate::models::NetworkShare>,
            crate::models::PaginatedResponse<crate::models::Note>,
            crate::models::PaginatedResponse<crate::models::Stack>,
            crate::models::PaginatedResponse<crate::models::Healthcheck>,
            
            // Applications
            crate::models::Application,
            crate::models::CreateApplication,
            crate::models::UpdateApplication,
            crate::models::ApplicationWithRelations,
            
            // Services
            crate::models::Service,
            crate::models::CreateService,
            crate::models::UpdateService,
            crate::models::ServiceRelation,
            crate::models::LinkService,
            crate::models::ServiceWithRelations,
            crate::models::ApplicationServiceRelation,
            
            // Infrastructure
            crate::models::Infra,
            crate::models::CreateInfra,
            crate::models::UpdateInfra,
            crate::models::InfraRelation,
            crate::models::LinkInfra,
            crate::models::InfraWithRelations,
            crate::models::ApplicationInfraRelation,
            crate::models::ServiceInfraRelation,
            
            // Domains
            crate::models::Domain,
            crate::models::CreateDomain,
            crate::models::UpdateDomain,
            crate::models::DomainRelation,
            crate::models::LinkDomain,
            crate::models::DomainTarget,
            crate::models::TargetName,
            crate::models::DomainWithRelations,
            crate::models::ApplicationDomainRelation,
            
            // People
            crate::models::Person,
            crate::models::CreatePerson,
            crate::models::UpdatePerson,
            crate::models::PersonRelation,
            crate::models::LinkPerson,
            crate::models::PersonWithRelations,
            crate::models::ApplicationPersonRelation,
            
            // Network shares
            crate::models::NetworkShare,
            crate::models::CreateNetworkShare,
            crate::models::UpdateNetworkShare,
            crate::models::NetworkShareRelation,
            crate::models::LinkNetworkShare,
            crate::models::NetworkShareWithRelations,
            crate::models::ApplicationNetworkShareRelation,
            
            // Notes
            crate::models::Note,
            crate::models::CreateNote,
            crate::models::UpdateNote,
            
            // Stacks
            crate::models::Stack,
            crate::models::CreateStack,
            crate::models::UpdateStack,
            crate::models::StackRelation,
            crate::models::StackWithRelations,
            crate::models::ApplicationStackRelation,
            
            // Healthchecks
            crate::models::Healthcheck,
            crate::models::CreateHealthcheck,
            crate::models::UpdateHealthcheck,
            crate::models::HealthcheckWithRelations,
            crate::models::HealthcheckRelation,
            crate::models::HealthcheckExecuteResult,
            crate::models::KumaMonitor,
            crate::models::KumaEndpoint,
            
            // Uptime
            crate::models::HeartbeatEntry,
            crate::models::MonitorUptime,
            crate::models::UptimeEvent,
            
            // Service layer
            crate::service::dashboard::DashboardStats,
            crate::service::dashboard::EntityStats,
            crate::service::dashboard::ExpiringDomain,
            crate::service::dashboard::HealthcheckStats,
            crate::service::dashboard::RecentActivity,
            crate::service::search::SearchResults,
            crate::service::search::SearchResult,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "applications", description = "Application management"),
        (name = "services", description = "Service management"),
        (name = "infra", description = "Infrastructure management"),
        (name = "domains", description = "Domain management"),
        (name = "people", description = "People management"),
        (name = "shares", description = "Network shares management"),
        (name = "notes", description = "Notes management"),
        (name = "stacks", description = "Technology stacks management"),
        (name = "healthchecks", description = "Health checks management"),
        (name = "dashboard", description = "Dashboard statistics"),
        (name = "search", description = "Global search"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "http_basic",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Basic)
                        .description(Some(
                            "HTTP Basic Authentication handled by Caddy reverse proxy",
                        ))
                        .build(),
                ),
            );
        }

        // Apply security globally to all operations
        openapi.security = Some(vec![utoipa::openapi::security::SecurityRequirement::new(
            "http_basic",
            Vec::<String>::new(),
        )]);
    }
}
