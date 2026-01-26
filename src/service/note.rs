use sqlx::SqlitePool;

use crate::models::{CreateNote, Note, PaginatedResponse, PaginationParams, UpdateNote, new_id};
use crate::{Error, Result};

pub async fn list_for_entity(
    pool: &SqlitePool,
    entity_type: &str,
    entity_id: &str,
    params: &PaginationParams,
) -> Result<PaginatedResponse<Note>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;

    let notes = sqlx::query_as::<_, Note>(
        r#"
        SELECT id, entity_type, entity_id, title, content, note_type, url, is_pinned, created_at, updated_at, created_by
        FROM note
        WHERE entity_type = ?1 AND entity_id = ?2
        ORDER BY is_pinned DESC, created_at DESC
        LIMIT ?3 OFFSET ?4
        "#,
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM note WHERE entity_type = ?1 AND entity_id = ?2")
            .bind(entity_type)
            .bind(entity_id)
            .fetch_one(pool)
            .await?;

    Ok(PaginatedResponse::new(notes, count.0, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Note> {
    sqlx::query_as::<_, Note>(
        r#"
        SELECT id, entity_type, entity_id, title, content, note_type, url, is_pinned, created_at, updated_at, created_by
        FROM note
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Note with id '{}' not found", id)))
}

pub async fn create(pool: &SqlitePool, input: CreateNote) -> Result<Note> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO note (id, entity_type, entity_id, title, content, note_type, url, is_pinned)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(&id)
    .bind(&input.entity_type)
    .bind(&input.entity_id)
    .bind(&input.title)
    .bind(&input.content)
    .bind(&input.note_type)
    .bind(&input.url)
    .bind(input.is_pinned)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: UpdateNote) -> Result<Note> {
    let existing = get(pool, id).await?;

    let title = input.title.unwrap_or(existing.title);
    let content = input.content.or(existing.content);
    let note_type = input.note_type.unwrap_or(existing.note_type);
    let url = input.url.or(existing.url);
    let is_pinned = input.is_pinned.unwrap_or(existing.is_pinned);

    sqlx::query(
        r#"
        UPDATE note
        SET title = ?1, content = ?2, note_type = ?3, url = ?4, is_pinned = ?5, updated_at = datetime('now')
        WHERE id = ?6
        "#,
    )
    .bind(&title)
    .bind(&content)
    .bind(&note_type)
    .bind(&url)
    .bind(is_pinned)
    .bind(id)
    .execute(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM note WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!("Note with id '{}' not found", id)));
    }

    Ok(())
}
