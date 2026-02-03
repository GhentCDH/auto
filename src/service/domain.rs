use futures::future::try_join_all;
use sqlx::SqlitePool;

use crate::models::{
    ApplicationDomainRelation, CreateDomain, Domain, DomainWithRelations,
    PaginatedResponse, PaginationParams, TargetName, UpdateDomain, new_id,
};
use crate::{Error, Result, service};

pub async fn list(
    pool: &SqlitePool,
    params: &PaginationParams,
) -> Result<PaginatedResponse<DomainWithRelations>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;
    let search_pattern = params.search.as_ref().map(|s| format!("%{}%", s));

    let domains = sqlx::query_as::<_, Domain>(
        r#"
        SELECT id, fqdn, registrar, dns_provider, expires_at, notes, 
            target_application_id, target_service_id, created_at, updated_at, created_by
        FROM domain
        WHERE (?1 IS NULL OR fqdn LIKE ?1 OR registrar LIKE ?1)
        ORDER BY fqdn ASC
        LIMIT ?2 OFFSET ?3
        "#,
    )
    .bind(&search_pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let domains = try_join_all(domains.into_iter().map(|d| extend_relations(pool, d))).await?;

    let (total,) = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*)
        FROM domain
        WHERE (?1 IS NULL OR fqdn LIKE ?1 OR registrar LIKE ?1)
        "#,
    )
    .bind(&search_pattern)
    .fetch_one(pool)
    .await?;

    Ok(PaginatedResponse::new(domains, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Domain> {
    sqlx::query_as::<_, Domain>(
        r#"
        SELECT id, fqdn, registrar, dns_provider, expires_at, notes, 
            target_application_id, target_service_id, created_at, updated_at, created_by
        FROM domain
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Domain with id '{}' not found", id)))
}

pub async fn extend_relations(pool: &SqlitePool, domain: Domain) -> Result<DomainWithRelations> {
    let applications = sqlx::query_as::<_, ApplicationDomainRelation>(
        r#"
        SELECT a.id, a.name
        FROM application a
        JOIN application_domain ad ON a.id = ad.application_id
        WHERE ad.domain_id = ?1
        ORDER BY a.name
        "#,
    )
    .bind(&domain.id)
    .fetch_all(pool)
    .await?;

    let target_application_name = if let Some(id) = &domain.target_application_id {
        Some(
            sqlx::query_as::<_, TargetName>(
                r#"
            SELECT a.name
            from application a
            where a.id = ?1
            "#,
            )
            .bind(id)
            .fetch_one(pool)
            .await?
            .name,
        )
    } else {
        None
    };

    let target_service_name = if let Some(id) = &domain.target_service_id {
        Some(
            sqlx::query_as::<_, TargetName>(
                r#"
            SELECT s.name
            from service s
            where s.id = ?1
            "#,
            )
            .bind(id)
            .fetch_one(pool)
            .await?
            .name,
        )
    } else {
        None
    };

    Ok(DomainWithRelations {
        domain,
        applications,
        target_application_name,
        target_service_name,
    })
}

pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<DomainWithRelations> {
    let domain = get(pool, id).await?;

    extend_relations(pool, domain).await
}

pub async fn create(pool: &SqlitePool, input: CreateDomain) -> Result<Domain> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO domain (id, fqdn, registrar, dns_provider, expires_at, notes, target_application_id, target_service_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(&id)
    .bind(&input.fqdn)
    .bind(&input.registrar)
    .bind(&input.dns_provider)
    .bind(&input.expires_at)
    .bind(&input.notes)
    .bind(&input.target_application_id)
    .bind(&input.target_service_id)
    .execute(pool)
    .await?;

    if let Some(app_id) = input.target_application_id {
        // link this domain to the application
        service::application::link_domain(pool, &app_id, &id, None).await?;
    }

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: UpdateDomain) -> Result<Domain> {
    let existing = get(pool, id).await?;

    let fqdn = input.fqdn.unwrap_or(existing.fqdn);
    let registrar = input.registrar.or(existing.registrar);
    let dns_provider = input.dns_provider.or(existing.dns_provider);
    let expires_at = input.expires_at.or(existing.expires_at);
    let notes = input.notes.or(existing.notes);
    let (target_application_id, target_service_id) =
        match (input.target_application_id, input.target_service_id) {
            (Some(app_id), None) => (Some(app_id), None),
            (None, Some(service_id)) => (None, Some(service_id)),
            (None, None) => (None, None),
            (Some(_), Some(_)) => {
                return Err(Error::ValidationError(
                    "can't set both target application and target service".into(),
                ));
            }
        };

    sqlx::query(
        r#"
        UPDATE domain
        SET fqdn = ?1, registrar = ?2, dns_provider = ?3, expires_at = ?4, notes = ?5, target_application_id = ?6, target_service_id = ?7, updated_at = datetime('now')
        WHERE id = ?8
        "#,
    )
    .bind(&fqdn)
    .bind(&registrar)
    .bind(&dns_provider)
    .bind(&expires_at)
    .bind(&notes)
    .bind(&target_application_id)
    .bind(&target_service_id)
    .bind(id)
    .execute(pool)
    .await?;

    if let Some(app_id) = target_application_id {
        // link this domain to the application
        service::application::link_domain(pool, &app_id, id, None).await?;
    } else if let Some(service_id) = target_service_id {
        // if this domain is linked to an application, and we're targetting a service, add that service
        // to the application
        let applications = sqlx::query_as::<_, ApplicationDomainRelation>(
            r#"
        SELECT a.id, a.name
        FROM application a
        JOIN application_domain ad ON a.id = ad.application_id
        WHERE ad.domain_id = ?1
        "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        for app in applications {
            service::application::link_service(
                pool,
                &app.id,
                &service_id,
                Some(&format!("through '{}'", fqdn)),
            )
            .await?;
        }
    }

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
