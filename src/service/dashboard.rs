use serde::Serialize;
use sqlx::SqlitePool;

use crate::Result;

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub applications: EntityStats,
    pub hosts: EntityStats,
    pub domains: EntityStats,
    pub people: EntityStats,
    pub clients: EntityStats,
    pub network_shares: EntityStats,
    pub notes: i64,
    pub expiring_domains: Vec<ExpiringDomain>,
    pub expiring_ssl: Vec<ExpiringSsl>,
}

#[derive(Debug, Serialize)]
pub struct EntityStats {
    pub total: i64,
    pub active: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ExpiringDomain {
    pub id: String,
    pub name: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ExpiringSsl {
    pub id: String,
    pub name: String,
    pub ssl_expires_at: Option<String>,
}

pub async fn get_stats(pool: &SqlitePool) -> Result<DashboardStats> {
    // Get application stats
    let app_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM application")
        .fetch_one(pool)
        .await?;
    let app_active: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM application WHERE status = 'active'")
            .fetch_one(pool)
            .await?;

    // Get host stats
    let host_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM host")
        .fetch_one(pool)
        .await?;
    let host_active: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM host WHERE status = 'active'")
        .fetch_one(pool)
        .await?;

    // Get domain stats
    let domain_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM domain")
        .fetch_one(pool)
        .await?;
    let domain_active: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM domain WHERE status = 'active'")
            .fetch_one(pool)
            .await?;

    // Get person stats
    let person_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM person")
        .fetch_one(pool)
        .await?;
    let person_active: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM person WHERE is_active = 1")
        .fetch_one(pool)
        .await?;

    // Get client stats
    let client_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM client")
        .fetch_one(pool)
        .await?;
    let client_active: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM client WHERE status = 'active'")
            .fetch_one(pool)
            .await?;

    // Get network share stats
    let share_total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM network_share")
        .fetch_one(pool)
        .await?;
    let share_active: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM network_share WHERE status = 'active'")
            .fetch_one(pool)
            .await?;

    // Get note count
    let note_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM note")
        .fetch_one(pool)
        .await?;

    // Get expiring domains (within 90 days)
    let expiring_domains = sqlx::query_as::<_, ExpiringDomain>(
        r#"
        SELECT id, name, expires_at
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

    // Get expiring SSL certificates (within 30 days)
    let expiring_ssl = sqlx::query_as::<_, ExpiringSsl>(
        r#"
        SELECT id, name, ssl_expires_at
        FROM domain
        WHERE ssl_expires_at IS NOT NULL
          AND date(ssl_expires_at) <= date('now', '+30 days')
          AND date(ssl_expires_at) >= date('now')
        ORDER BY ssl_expires_at ASC
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
        hosts: EntityStats {
            total: host_total.0,
            active: host_active.0,
        },
        domains: EntityStats {
            total: domain_total.0,
            active: domain_active.0,
        },
        people: EntityStats {
            total: person_total.0,
            active: person_active.0,
        },
        clients: EntityStats {
            total: client_total.0,
            active: client_active.0,
        },
        network_shares: EntityStats {
            total: share_total.0,
            active: share_active.0,
        },
        notes: note_count.0,
        expiring_domains,
        expiring_ssl,
    })
}
