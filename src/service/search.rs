use serde::Serialize;
use sqlx::SqlitePool;

use crate::Result;

#[derive(Debug, Serialize)]
pub struct SearchResults {
    pub applications: Vec<SearchResult>,
    pub hosts: Vec<SearchResult>,
    pub domains: Vec<SearchResult>,
    pub people: Vec<SearchResult>,
    pub network_shares: Vec<SearchResult>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
}

pub async fn global_search(pool: &SqlitePool, query: &str) -> Result<SearchResults> {
    let search_pattern = format!("%{}%", query);

    let applications = sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, name, description, 'application' as entity_type
        FROM application
        WHERE name LIKE ?1 OR description LIKE ?1
        ORDER BY name ASC
        LIMIT 20
        "#,
    )
    .bind(&search_pattern)
    .fetch_all(pool)
    .await?;

    let hosts = sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, name, hostname as description, 'host' as entity_type
        FROM host
        WHERE name LIKE ?1 OR hostname LIKE ?1 OR ip_address LIKE ?1
        ORDER BY name ASC
        LIMIT 20
        "#,
    )
    .bind(&search_pattern)
    .fetch_all(pool)
    .await?;

    let domains = sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, name, registrar as description, 'domain' as entity_type
        FROM domain
        WHERE name LIKE ?1 OR registrar LIKE ?1
        ORDER BY name ASC
        LIMIT 20
        "#,
    )
    .bind(&search_pattern)
    .fetch_all(pool)
    .await?;

    let people = sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, name, email as description, 'person' as entity_type
        FROM person
        WHERE name LIKE ?1 OR email LIKE ?1 OR role LIKE ?1
        ORDER BY name ASC
        LIMIT 20
        "#,
    )
    .bind(&search_pattern)
    .fetch_all(pool)
    .await?;

    let network_shares = sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, name, path as description, 'network_share' as entity_type
        FROM network_share
        WHERE name LIKE ?1 OR path LIKE ?1 OR server LIKE ?1
        ORDER BY name ASC
        LIMIT 20
        "#,
    )
    .bind(&search_pattern)
    .fetch_all(pool)
    .await?;

    Ok(SearchResults {
        applications,
        hosts,
        domains,
        people,
        network_shares,
    })
}
