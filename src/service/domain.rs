use sqlx::SqlitePool;

use crate::models::{
    new_id, ApplicationDomainRelation, CreateDomain, Domain, DomainWithRelations, PaginatedResponse,
    PaginationParams, UpdateDomain,
};
use crate::{Error, Result};

pub async fn list(pool: &SqlitePool, params: &PaginationParams) -> Result<PaginatedResponse<Domain>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;

    let (domains, total): (Vec<Domain>, i64) = if let Some(search) = &params.search {
        let search_pattern = format!("%{}%", search);
        let items = sqlx::query_as::<_, Domain>(
            r#"
            SELECT id, name, registrar, dns_provider, expires_at, ssl_expires_at, ssl_issuer, status, notes, created_at, updated_at, created_by
            FROM domain
            WHERE name LIKE ?1 OR registrar LIKE ?1
            ORDER BY name ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM domain WHERE name LIKE ?1 OR registrar LIKE ?1",
        )
        .bind(&search_pattern)
        .fetch_one(pool)
        .await?;

        (items, count.0)
    } else {
        let items = sqlx::query_as::<_, Domain>(
            r#"
            SELECT id, name, registrar, dns_provider, expires_at, ssl_expires_at, ssl_issuer, status, notes, created_at, updated_at, created_by
            FROM domain
            ORDER BY name ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM domain")
            .fetch_one(pool)
            .await?;

        (items, count.0)
    };

    Ok(PaginatedResponse::new(domains, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Domain> {
    sqlx::query_as::<_, Domain>(
        r#"
        SELECT id, name, registrar, dns_provider, expires_at, ssl_expires_at, ssl_issuer, status, notes, created_at, updated_at, created_by
        FROM domain
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Domain with id '{}' not found", id)))
}

pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<DomainWithRelations> {
    let domain = get(pool, id).await?;

    let applications = sqlx::query_as::<_, ApplicationDomainRelation>(
        r#"
        SELECT a.id, a.name, a.status, ad.record_type, ad.is_primary
        FROM application a
        JOIN application_domain ad ON a.id = ad.application_id
        WHERE ad.domain_id = ?1
        ORDER BY a.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(DomainWithRelations { domain, applications })
}

pub async fn create(pool: &SqlitePool, input: CreateDomain) -> Result<Domain> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO domain (id, name, registrar, dns_provider, expires_at, ssl_expires_at, ssl_issuer, status, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.registrar)
    .bind(&input.dns_provider)
    .bind(&input.expires_at)
    .bind(&input.ssl_expires_at)
    .bind(&input.ssl_issuer)
    .bind(&input.status)
    .bind(&input.notes)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: UpdateDomain) -> Result<Domain> {
    let existing = get(pool, id).await?;

    let name = input.name.unwrap_or(existing.name);
    let registrar = input.registrar.or(existing.registrar);
    let dns_provider = input.dns_provider.or(existing.dns_provider);
    let expires_at = input.expires_at.or(existing.expires_at);
    let ssl_expires_at = input.ssl_expires_at.or(existing.ssl_expires_at);
    let ssl_issuer = input.ssl_issuer.or(existing.ssl_issuer);
    let status = input.status.unwrap_or(existing.status);
    let notes = input.notes.or(existing.notes);

    sqlx::query(
        r#"
        UPDATE domain
        SET name = ?1, registrar = ?2, dns_provider = ?3, expires_at = ?4, ssl_expires_at = ?5, ssl_issuer = ?6, status = ?7, notes = ?8, updated_at = datetime('now')
        WHERE id = ?9
        "#,
    )
    .bind(&name)
    .bind(&registrar)
    .bind(&dns_provider)
    .bind(&expires_at)
    .bind(&ssl_expires_at)
    .bind(&ssl_issuer)
    .bind(&status)
    .bind(&notes)
    .bind(id)
    .execute(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM domain WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!("Domain with id '{}' not found", id)));
    }

    Ok(())
}
