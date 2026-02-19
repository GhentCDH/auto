use serde::Serialize;
use sqlx::SqlitePool;
use tokio::try_join;
use utoipa::ToSchema;

use crate::Result;

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResults {
    pub applications: Vec<SearchResult>,
    pub services: Vec<SearchResult>,
    pub infra: Vec<SearchResult>,
    pub domains: Vec<SearchResult>,
    pub people: Vec<SearchResult>,
    pub network_shares: Vec<SearchResult>,
    pub stacks: Vec<SearchResult>,
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
    let (applications, services, infra, domains, people, network_shares, stacks) = try_join!(
        search_applications(pool, &pattern, &prefix),
        search_services(pool, &pattern, &prefix),
        search_infra(pool, &pattern, &prefix),
        search_domains(pool, &pattern, &prefix),
        search_people(pool, &pattern, &prefix),
        search_network_shares(pool, &pattern, &prefix),
        search_stacks(pool, &pattern, &prefix),
    )?;

    Ok(SearchResults {
        applications,
        services,
        infra,
        domains,
        people,
        network_shares,
        stacks,
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
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name ASC
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
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name ASC
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
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name ASC
        LIMIT 20
        "#,
    )
    .bind(pattern)
    .bind(prefix)
    .fetch_all(pool)
    .await?)
}

async fn search_domains(
    pool: &SqlitePool,
    pattern: &str,
    prefix: &str,
) -> Result<Vec<SearchResult>> {
    Ok(sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, fqdn as name, registrar as description, 'domain' as entity_type
        FROM domain
        WHERE fqdn LIKE ?1 OR registrar LIKE ?1
        ORDER BY CASE WHEN fqdn LIKE ?2 THEN 0 ELSE 1 END, fqdn ASC
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
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name ASC
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
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name ASC
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
        ORDER BY CASE WHEN name LIKE ?2 THEN 0 ELSE 1 END, name ASC
        LIMIT 20
        "#,
    )
    .bind(pattern)
    .bind(prefix)
    .fetch_all(pool)
    .await?)
}
