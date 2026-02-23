use sqlx::SqlitePool;

use crate::models::{
    ApplicationInfraRelation, CreateInfra, Infra, InfraWithRelations, PaginatedResponse,
    PaginationParams, ServiceInfraRelation, UpdateInfra, new_id,
};
use crate::{Error, Result};

pub async fn list(
    pool: &SqlitePool,
    params: &PaginationParams,
    infra_type: Option<&str>,
) -> Result<PaginatedResponse<Infra>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;
    let search_pattern = params.search.as_ref().map(|s| format!("%{}%", s));

    let items = sqlx::query_as::<_, Infra>(
        r#"
        SELECT id, name, description, type, created_at, updated_at, created_by
        FROM infra
        WHERE (?1 IS NULL OR name LIKE ?1 OR description LIKE ?1)
          AND (?2 IS NULL OR type = ?2)
        ORDER BY name COLLATE NOCASE ASC
        LIMIT ?3 OFFSET ?4
        "#,
    )
    .bind(&search_pattern)
    .bind(infra_type)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let (total,) = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*)
        FROM infra
        WHERE (?1 IS NULL OR name LIKE ?1 OR description LIKE ?1)
          AND (?2 IS NULL OR type = ?2)
        "#,
    )
    .bind(&search_pattern)
    .bind(infra_type)
    .fetch_one(pool)
    .await?;

    Ok(PaginatedResponse::new(items, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Infra> {
    sqlx::query_as::<_, Infra>(
        r#"
        SELECT id, name, description, type, created_at, updated_at, created_by
        FROM infra
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Infra with id '{}' not found", id)))
}

pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<InfraWithRelations> {
    let infra = get(pool, id).await?;

    let applications = sqlx::query_as::<_, ApplicationInfraRelation>(
        r#"
        SELECT a.id, a.name, a.environment, a.status
        FROM application a
        JOIN application_infra ai ON a.id = ai.application_id
        WHERE ai.infra_id = ?1
        ORDER BY a.name COLLATE NOCASE
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let services = sqlx::query_as::<_, ServiceInfraRelation>(
        r#"
        SELECT s.id, s.name, s.environment, s.status
        FROM service s
        JOIN service_infra si ON s.id = si.service_id
        WHERE si.infra_id = ?1
        ORDER BY s.name COLLATE NOCASE
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(InfraWithRelations {
        infra,
        applications,
        services,
    })
}

pub async fn create(pool: &SqlitePool, input: CreateInfra) -> Result<Infra> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO infra (id, name, description, type)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.infra_type)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: UpdateInfra) -> Result<Infra> {
    let existing = get(pool, id).await?;

    let name = input.name.unwrap_or(existing.name);
    let description = input.description.or(existing.description);
    let infra_type = input.infra_type.unwrap_or(existing.infra_type);

    sqlx::query(
        r#"
        UPDATE infra
        SET name = ?1, description = ?2, type = ?3, updated_at = datetime('now')
        WHERE id = ?4
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(&infra_type)
    .bind(id)
    .execute(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM infra WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!("Infra with id '{}' not found", id)));
    }

    Ok(())
}
