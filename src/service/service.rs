use sqlx::SqlitePool;

use crate::models::{
    ApplicationServiceRelation, CreateService, InfraRelation, PaginatedResponse, PaginationParams,
    Service, ServiceWithRelations, UpdateService, new_id,
};
use crate::{Error, Result, service};

pub async fn list(
    pool: &SqlitePool,
    params: &PaginationParams,
    status: Option<&str>,
    environment: Option<&str>,
) -> Result<PaginatedResponse<Service>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;
    let search_pattern = params.search.as_ref().map(|s| format!("%{}%", s));

    let services = sqlx::query_as::<_, Service>(
        r#"
        SELECT id, name, description, repository_url, environment, status, image_refs, created_at, updated_at, created_by
        FROM service
        WHERE (?1 IS NULL OR name LIKE ?1 OR description LIKE ?1)
          AND (?2 IS NULL OR status = ?2)
          AND (?3 IS NULL OR environment = ?3)
        ORDER BY name COLLATE NOCASE ASC
        LIMIT ?4 OFFSET ?5
        "#,
    )
    .bind(&search_pattern)
    .bind(status)
    .bind(environment)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let (total,) = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*)
        FROM service
        WHERE (?1 IS NULL OR name LIKE ?1 OR description LIKE ?1)
          AND (?2 IS NULL OR status = ?2)
          AND (?3 IS NULL OR environment = ?3)
        "#,
    )
    .bind(&search_pattern)
    .bind(status)
    .bind(environment)
    .fetch_one(pool)
    .await?;

    Ok(PaginatedResponse::new(services, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Service> {
    sqlx::query_as::<_, Service>(
        r#"
        SELECT id, name, description, repository_url, environment, status, image_refs, created_at, updated_at, created_by
        FROM service
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Service with id '{}' not found", id)))
}

pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<ServiceWithRelations> {
    let service = get(pool, id).await?;

    let applications = sqlx::query_as::<_, ApplicationServiceRelation>(
        r#"
        SELECT a.id, a.name, a.environment, a.status
        FROM application a
        JOIN application_service asvc ON a.id = asvc.application_id
        WHERE asvc.service_id = ?1
        ORDER BY a.name COLLATE NOCASE
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let infra = sqlx::query_as::<_, InfraRelation>(
        r#"
        SELECT i.id, i.name, i.type, si.notes as relation_notes
        FROM infra i
        JOIN service_infra si ON i.id = si.infra_id
        WHERE si.service_id = ?1
        ORDER BY i.name COLLATE NOCASE
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let healthchecks = service::healthcheck::get_for_service(pool, id).await?;

    Ok(ServiceWithRelations {
        service,
        applications,
        infra,
        healthchecks,
    })
}

pub async fn create(pool: &SqlitePool, input: CreateService) -> Result<Service> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO service (id, name, description, repository_url, environment, status, image_refs)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.repository_url)
    .bind(&input.environment)
    .bind(&input.status)
    .bind(&input.image_refs)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: UpdateService) -> Result<Service> {
    let existing = get(pool, id).await?;

    let name = input.name.unwrap_or(existing.name);
    let description = input.description.or(existing.description);
    let repository_url = input.repository_url.or(existing.repository_url);
    let environment = input.environment.unwrap_or(existing.environment);
    let status = input.status.unwrap_or(existing.status);
    let image_refs = input.image_refs.or(existing.image_refs);

    sqlx::query(
        r#"
        UPDATE service
        SET name = ?1, description = ?2, repository_url = ?3, environment = ?4, status = ?5, image_refs = ?6, updated_at = datetime('now')
        WHERE id = ?7
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(&repository_url)
    .bind(&environment)
    .bind(&status)
    .bind(&image_refs)
    .bind(id)
    .execute(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM service WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!(
            "Service with id '{}' not found",
            id
        )));
    }

    Ok(())
}

// Relationship management - link/unlink infra

pub async fn link_infra(
    pool: &SqlitePool,
    service_id: &str,
    infra_id: &str,
    notes: Option<&str>,
) -> Result<()> {
    // Verify both entities exist
    get(pool, service_id).await?;
    crate::service::infra::get(pool, infra_id).await?;

    sqlx::query(
        r#"
        INSERT INTO service_infra (service_id, infra_id, notes)
        VALUES (?1, ?2, ?3)
        ON CONFLICT (service_id, infra_id) DO UPDATE SET notes = ?3, updated_at = datetime('now')
        "#,
    )
    .bind(service_id)
    .bind(infra_id)
    .bind(notes)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unlink_infra(pool: &SqlitePool, service_id: &str, infra_id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM service_infra WHERE service_id = ?1 AND infra_id = ?2")
        .bind(service_id)
        .bind(infra_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound("Relationship not found".to_string()));
    }

    Ok(())
}
