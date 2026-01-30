use sqlx::SqlitePool;

use crate::models::{
    ApplicationStackRelation, CreateStack, PaginatedResponse, PaginationParams, Stack,
    StackWithRelations, UpdateStack, new_id,
};
use crate::{Error, Result};

pub async fn list(
    pool: &SqlitePool,
    params: &PaginationParams,
) -> Result<PaginatedResponse<Stack>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;

    let (stacks, total): (Vec<Stack>, i64) = if let Some(search) = &params.search {
        let search_pattern = format!("%{}%", search);
        let items = sqlx::query_as::<_, Stack>(
            r#"
            SELECT id, name, notes, created_at, updated_at
            FROM stack
            WHERE name LIKE ?1
            ORDER BY name ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM stack WHERE name LIKE ?1")
            .bind(&search_pattern)
            .fetch_one(pool)
            .await?;

        (items, count.0)
    } else {
        let items = sqlx::query_as::<_, Stack>(
            r#"
            SELECT id, name, notes, created_at, updated_at
            FROM stack
            ORDER BY name ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM stack")
            .fetch_one(pool)
            .await?;

        (items, count.0)
    };

    Ok(PaginatedResponse::new(stacks, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Stack> {
    sqlx::query_as::<_, Stack>(
        r#"
        SELECT id, name, notes, created_at, updated_at
        FROM stack
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Stack with id '{}' not found", id)))
}

pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<StackWithRelations> {
    let stack = get(pool, id).await?;

    let applications = sqlx::query_as::<_, ApplicationStackRelation>(
        r#"
        SELECT a.id, a.name, a.status
        FROM application a
        JOIN application_stack ast ON a.id = ast.application_id
        WHERE ast.stack_id = ?1
        ORDER BY a.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(StackWithRelations {
        stack,
        applications,
    })
}

pub async fn create(pool: &SqlitePool, input: CreateStack) -> Result<Stack> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO stack (id, name, notes)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.notes)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: UpdateStack) -> Result<Stack> {
    let existing = get(pool, id).await?;

    let name = input.name.unwrap_or(existing.name);
    let notes = input.notes.or(existing.notes);

    sqlx::query(
        r#"
        UPDATE stack
        SET name = ?1, notes = ?2, updated_at = datetime('now')
        WHERE id = ?3
        "#,
    )
    .bind(&name)
    .bind(&notes)
    .bind(id)
    .execute(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM stack WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!("Stack with id '{}' not found", id)));
    }

    Ok(())
}
