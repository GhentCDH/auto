use serde::Serialize;
use sqlx::SqlitePool;
use tokio::try_join;
use utoipa::ToSchema;

use crate::{Error, Result};

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct ResolvedEntity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
}

/// Look up which entity type owns the given UUID (or UUID prefix).
pub async fn resolve_id(pool: &SqlitePool, id: &str) -> Result<ResolvedEntity> {
    let pattern = format!("{id}%");
    let result = sqlx::query_as::<_, ResolvedEntity>(
        r#"
        SELECT id, name, 'application' as entity_type FROM application WHERE id LIKE ?1
        UNION ALL SELECT id, name, 'service' FROM service WHERE id LIKE ?1
        UNION ALL SELECT id, name, 'infra' FROM infra WHERE id LIKE ?1
        UNION ALL SELECT id, fqdn as name, 'domain' FROM domain WHERE id LIKE ?1
        UNION ALL SELECT id, name, 'person' FROM person WHERE id LIKE ?1
        UNION ALL SELECT id, name, 'network_share' FROM network_share WHERE id LIKE ?1
        UNION ALL SELECT id, name, 'stack' FROM stack WHERE id LIKE ?1
        UNION ALL SELECT id, name, 'healthcheck' FROM healthcheck WHERE id LIKE ?1
        LIMIT 1
        "#,
    )
    .bind(&pattern)
    .fetch_optional(pool)
    .await?;

    result.ok_or(Error::NotFound(format!("No entity found with id {id}")))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResults {
    pub applications: Vec<SearchResult>,
    pub services: Vec<SearchResult>,
    pub infra: Vec<SearchResult>,
    pub domains: Vec<SearchResult>,
    pub people: Vec<SearchResult>,
    pub network_shares: Vec<SearchResult>,
    pub stacks: Vec<SearchResult>,
    pub healthchecks: Vec<SearchResult>,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
}

pub async fn global_search(pool: &SqlitePool, query: &str) -> Result<SearchResults> {
    let pattern = format!("%{}%", query);
    let prefix = format!("{}%", query);

    // Run search queries concurrently!
    let (applications, services, infra, domains, people, network_shares, stacks, healthchecks) = try_join!(
        search_applications(pool, &pattern, &prefix),
        search_services(pool, &pattern, &prefix),
        search_infra(pool, &pattern, &prefix),
        search_domains(pool, &pattern, &prefix),
        search_people(pool, &pattern, &prefix),
        search_network_shares(pool, &pattern, &prefix),
        search_stacks(pool, &pattern, &prefix),
        search_healthchecks(pool, &pattern, &prefix),
    )?;

    Ok(SearchResults {
        applications,
        services,
        infra,
        domains,
        people,
        network_shares,
        stacks,
        healthchecks,
    })
}

async fn search_applications(
    pool: &SqlitePool,
    pattern: &str,
    prefix: &str,
) -> Result<Vec<SearchResult>> {
    Ok(sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, name, description, 'application' as entity_type
        FROM application
        WHERE name LIKE ?1 OR description LIKE ?1
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name COLLATE NOCASE ASC
        LIMIT 20
        "#,
    )
    .bind(pattern)
    .bind(prefix)
    .fetch_all(pool)
    .await?)
}

async fn search_services(
    pool: &SqlitePool,
    pattern: &str,
    prefix: &str,
) -> Result<Vec<SearchResult>> {
    Ok(sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, name, description, 'service' as entity_type
        FROM service
        WHERE name LIKE ?1 OR description LIKE ?1
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name COLLATE NOCASE ASC
        LIMIT 20
        "#,
    )
    .bind(pattern)
    .bind(prefix)
    .fetch_all(pool)
    .await?)
}

async fn search_infra(pool: &SqlitePool, pattern: &str, prefix: &str) -> Result<Vec<SearchResult>> {
    Ok(sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, name, description, 'infra' as entity_type
        FROM infra
        WHERE name LIKE ?1 OR description LIKE ?1
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name COLLATE NOCASE ASC
        LIMIT 20
        "#,
    )
    .bind(pattern)
    .bind(prefix)
    .fetch_all(pool)
    .await?)
}

/// Search domains directly AND via linked applications/services.
/// When a search term matches an application or service name, domains
/// targeting that application/service are included in results.
async fn search_domains(
    pool: &SqlitePool,
    pattern: &str,
    prefix: &str,
) -> Result<Vec<SearchResult>> {
    Ok(sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT DISTINCT d.id, d.fqdn as name, d.registrar as description, 'domain' as entity_type
        FROM domain d
        LEFT JOIN application a ON d.target_application_id = a.id
        LEFT JOIN service s ON d.target_service_id = s.id
        WHERE d.fqdn LIKE ?1
           OR d.registrar LIKE ?1
           OR d.dns_provider LIKE ?1
           OR d.notes LIKE ?1
           OR a.name LIKE ?1
           OR s.name LIKE ?1
        ORDER BY CASE WHEN d.fqdn LIKE ?2 THEN 0 ELSE 1 END, d.fqdn COLLATE NOCASE ASC
        LIMIT 20
        "#,
    )
    .bind(pattern)
    .bind(prefix)
    .fetch_all(pool)
    .await?)
}

async fn search_people(
    pool: &SqlitePool,
    pattern: &str,
    prefix: &str,
) -> Result<Vec<SearchResult>> {
    Ok(sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, name, email as description, 'person' as entity_type
        FROM person
        WHERE name LIKE ?1 OR email LIKE ?1 OR role LIKE ?1
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name COLLATE NOCASE ASC
        LIMIT 20
        "#,
    )
    .bind(pattern)
    .bind(prefix)
    .fetch_all(pool)
    .await?)
}

async fn search_network_shares(
    pool: &SqlitePool,
    pattern: &str,
    prefix: &str,
) -> Result<Vec<SearchResult>> {
    Ok(sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, name, path as description, 'network_share' as entity_type
        FROM network_share
        WHERE name LIKE ?1 OR path LIKE ?1 OR server LIKE ?1
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name COLLATE NOCASE ASC
        LIMIT 20
        "#,
    )
    .bind(pattern)
    .bind(prefix)
    .fetch_all(pool)
    .await?)
}

async fn search_stacks(
    pool: &SqlitePool,
    pattern: &str,
    prefix: &str,
) -> Result<Vec<SearchResult>> {
    Ok(sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, name, notes as description, 'stack' as entity_type
        FROM stack
        WHERE name LIKE ?1 OR notes LIKE ?1
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name COLLATE NOCASE ASC
        LIMIT 20
        "#,
    )
    .bind(pattern)
    .bind(prefix)
    .fetch_all(pool)
    .await?)
}

async fn search_healthchecks(
    pool: &SqlitePool,
    pattern: &str,
    prefix: &str,
) -> Result<Vec<SearchResult>> {
    Ok(sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT h.id, h.name,
               h.protocol || '://' || d.fqdn || h.path as description,
               'healthcheck' as entity_type
        FROM healthcheck h
        JOIN domain d ON h.domain_id = d.id
        LEFT JOIN application a ON h.application_id = a.id
        LEFT JOIN service s ON h.service_id = s.id
        WHERE h.name LIKE ?1
           OR d.fqdn LIKE ?1
           OR h.path LIKE ?1
           OR h.notes LIKE ?1
           OR a.name LIKE ?1
           OR s.name LIKE ?1
        ORDER BY CASE WHEN h.name LIKE ?2 THEN 0 ELSE 1 END, h.name COLLATE NOCASE ASC
        LIMIT 20
        "#,
    )
    .bind(pattern)
    .bind(prefix)
    .fetch_all(pool)
    .await?)
}
