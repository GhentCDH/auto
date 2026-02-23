use serde::Serialize;
use sqlx::SqlitePool;
use tracing::info;
use utoipa::ToSchema;

use crate::Result;

#[derive(Debug, Serialize, ToSchema)]
pub struct DashboardStats {
    pub applications: EntityStats,
    pub services: EntityStats,
    pub infra: EntityStats,
    pub domains: EntityStats,
    pub people: EntityStats,
    pub network_shares: EntityStats,
    pub notes: i64,
    pub expiring_domains: Vec<ExpiringDomain>,
    pub healthchecks: HealthcheckStats,
    pub recent_activity: Vec<RecentActivity>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthcheckStats {
    pub total: i64,
    pub enabled: i64,
    pub kuma_dirty: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct RecentActivity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EntityStats {
    pub total: i64,
    pub active: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct ExpiringDomain {
    pub id: String,
    pub fqdn: String,
    pub expires_at: Option<String>,
}

pub async fn get_stats(pool: &SqlitePool) -> Result<DashboardStats> {
    // Get application stats
    info!("Application stats");
    let app_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM application")
        .fetch_one(pool)
        .await?;
    let app_active: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM application WHERE status = 'active'")
            .fetch_one(pool)
            .await?;

    // Get service stats
    info!("Service stats");
    let service_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM service")
        .fetch_one(pool)
        .await?;
    let service_active: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM service WHERE status = 'active'")
            .fetch_one(pool)
            .await?;

    // Get infra stats
    info!("Infra stats");
    let infra_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM infra")
        .fetch_one(pool)
        .await?;

    // Get domain stats
    info!("Domain stats");
    let domain_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM domain")
        .fetch_one(pool)
        .await?;

    // Get person stats
    info!("Person stats");
    let person_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM person")
        .fetch_one(pool)
        .await?;
    let person_active: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM person WHERE is_active = 1")
        .fetch_one(pool)
        .await?;

    // Get network share stats
    info!("Network stats");
    let share_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM network_share")
        .fetch_one(pool)
        .await?;
    let share_active: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM network_share WHERE status = 'active'")
            .fetch_one(pool)
            .await?;

    // Get note count
    info!("Note stats");
    let note_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM note")
        .fetch_one(pool)
        .await?;

    // Get healthcheck stats
    info!("Healthcheck stats");
    let hc_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM healthcheck")
        .fetch_one(pool)
        .await?;
    let hc_enabled: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM healthcheck WHERE is_enabled = 1")
            .fetch_one(pool)
            .await?;
    let hc_dirty: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM healthcheck WHERE kuma_dirty = 1")
        .fetch_one(pool)
        .await?;

    // Get recent activity across all entity types
    info!("Recent activity");
    let recent_activity = sqlx::query_as::<_, RecentActivity>(
        r#"
        SELECT id, name, 'application' as entity_type, updated_at FROM application
        UNION ALL
        SELECT id, name, 'service' as entity_type, updated_at FROM service
        UNION ALL
        SELECT id, name, 'infra' as entity_type, updated_at FROM infra
        UNION ALL
        SELECT id, fqdn as name, 'domain' as entity_type, updated_at FROM domain
        UNION ALL
        SELECT id, name, 'person' as entity_type, updated_at FROM person
        UNION ALL
        SELECT id, name, 'network_share' as entity_type, updated_at FROM network_share
        UNION ALL
        SELECT id, name, 'healthcheck' as entity_type, updated_at FROM healthcheck
        ORDER BY updated_at DESC
        LIMIT 15
        "#,
    )
    .fetch_all(pool)
    .await?;

    // Get expiring domains (within 90 days)
    let expiring_domains = sqlx::query_as::<_, ExpiringDomain>(
        r#"
        SELECT id, fqdn, expires_at
        FROM domain
        WHERE expires_at IS NOT NULL
          AND date(expires_at) <= date('now', '+90 days')
          AND date(expires_at) >= date('now')
        ORDER BY expires_at ASC 
        LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(DashboardStats {
        applications: EntityStats {
            total: app_total.0,
            active: app_active.0,
        },
        services: EntityStats {
            total: service_total.0,
            active: service_active.0,
        },
        infra: EntityStats {
            total: infra_total.0,
            active: infra_total.0, // Infra doesn't have status
        },
        domains: EntityStats {
            total: domain_total.0,
            active: domain_total.0, // Domain doesn't have status
        },
        people: EntityStats {
            total: person_total.0,
            active: person_active.0,
        },
        network_shares: EntityStats {
            total: share_total.0,
            active: share_active.0,
        },
        notes: note_count.0,
        expiring_domains,
        healthchecks: HealthcheckStats {
            total: hc_total.0,
            enabled: hc_enabled.0,
            kuma_dirty: hc_dirty.0,
        },
        recent_activity,
    })
}
