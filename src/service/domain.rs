use sqlx::SqlitePool;

use crate::models::{
    ApplicationDomainRelation, CreateDomain, Domain, DomainWithRelations, PaginatedResponse,
    PaginationParams, UpdateDomain, new_id,
};
use crate::{Error, Result};

pub async fn list(
    pool: &SqlitePool,
    params: &PaginationParams,
    status: Option<&str>,
) -> Result<PaginatedResponse<Domain>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;
    let search_pattern = params.search.as_ref().map(|s| format!("%{}%", s));

    let domains = sqlx::query_as::<_, Domain>(
        r#"
        SELECT id, name, registrar, dns_provider, expires_at, status, notes, created_at, updated_at, created_by
        FROM domain
        WHERE (?1 IS NULL OR name LIKE ?1 OR registrar LIKE ?1)
          AND (?2 IS NULL OR status = ?2)
        ORDER BY name ASC
        LIMIT ?3 OFFSET ?4
        "#,
    )
    .bind(&search_pattern)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let (total,) = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*)
        FROM domain
        WHERE (?1 IS NULL OR name LIKE ?1 OR registrar LIKE ?1)
          AND (?2 IS NULL OR status = ?2)
        "#,
    )
    .bind(&search_pattern)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(PaginatedResponse::new(domains, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Domain> {
    sqlx::query_as::<_, Domain>(
        r#"
        SELECT id, name, registrar, dns_provider, expires_at, status, notes, created_at, updated_at, created_by
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

    Ok(DomainWithRelations {
        domain,
        applications,
    })
}

pub async fn create(pool: &SqlitePool, input: CreateDomain) -> Result<Domain> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO domain (id, name, registrar, dns_provider, expires_at, status, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.registrar)
    .bind(&input.dns_provider)
    .bind(&input.expires_at)
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
    let status = input.status.unwrap_or(existing.status);
    let notes = input.notes.or(existing.notes);

    sqlx::query(
        r#"
        UPDATE domain
        SET name = ?1, registrar = ?2, dns_provider = ?3, expires_at = ?4, status = ?5, notes = ?6, updated_at = datetime('now')
        WHERE id = ?9
        "#,
    )
    .bind(&name)
    .bind(&registrar)
    .bind(&dns_provider)
    .bind(&expires_at)
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
        return Err(Error::NotFound(format!(
            "Domain with id '{}' not found",
            id
        )));
    }

    Ok(())
}
