use sqlx::SqlitePool;

use crate::models::{
    ApplicationNetworkShareRelation, CreateNetworkShare, NetworkShare, NetworkShareWithRelations,
    PaginatedResponse, PaginationParams, UpdateNetworkShare, new_id,
};
use crate::{Error, Result};

pub async fn list(
    pool: &SqlitePool,
    params: &PaginationParams,
) -> Result<PaginatedResponse<NetworkShare>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;

    let (shares, total): (Vec<NetworkShare>, i64) = if let Some(search) = &params.search {
        let search_pattern = format!("%{}%", search);
        let items = sqlx::query_as::<_, NetworkShare>(
            r#"
            SELECT id, name, path, share_type, server, purpose, status, notes, created_at, updated_at, created_by
            FROM network_share
            WHERE name LIKE ?1 OR path LIKE ?1 OR server LIKE ?1
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
            "SELECT COUNT(*) FROM network_share WHERE name LIKE ?1 OR path LIKE ?1 OR server LIKE ?1",
        )
        .bind(&search_pattern)
        .fetch_one(pool)
        .await?;

        (items, count.0)
    } else {
        let items = sqlx::query_as::<_, NetworkShare>(
            r#"
            SELECT id, name, path, share_type, server, purpose, status, notes, created_at, updated_at, created_by
            FROM network_share
            ORDER BY name ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM network_share")
            .fetch_one(pool)
            .await?;

        (items, count.0)
    };

    Ok(PaginatedResponse::new(shares, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<NetworkShare> {
    sqlx::query_as::<_, NetworkShare>(
        r#"
        SELECT id, name, path, share_type, server, purpose, status, notes, created_at, updated_at, created_by
        FROM network_share
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Network share with id '{}' not found", id)))
}

pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<NetworkShareWithRelations> {
    let network_share = get(pool, id).await?;

    let applications = sqlx::query_as::<_, ApplicationNetworkShareRelation>(
        r#"
        SELECT a.id, a.name, a.status, ans.usage, ans.mount_point
        FROM application a
        JOIN application_network_share ans ON a.id = ans.application_id
        WHERE ans.network_share_id = ?1
        ORDER BY a.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(NetworkShareWithRelations {
        network_share,
        applications,
    })
}

pub async fn create(pool: &SqlitePool, input: CreateNetworkShare) -> Result<NetworkShare> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO network_share (id, name, path, share_type, server, purpose, status, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.path)
    .bind(&input.share_type)
    .bind(&input.server)
    .bind(&input.purpose)
    .bind(&input.status)
    .bind(&input.notes)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: UpdateNetworkShare,
) -> Result<NetworkShare> {
    let existing = get(pool, id).await?;

    let name = input.name.unwrap_or(existing.name);
    let path = input.path.unwrap_or(existing.path);
    let share_type = input.share_type.unwrap_or(existing.share_type);
    let server = input.server.or(existing.server);
    let purpose = input.purpose.or(existing.purpose);
    let status = input.status.unwrap_or(existing.status);
    let notes = input.notes.or(existing.notes);

    sqlx::query(
        r#"
        UPDATE network_share
        SET name = ?1, path = ?2, share_type = ?3, server = ?4, purpose = ?5, status = ?6, notes = ?7, updated_at = datetime('now')
        WHERE id = ?8
        "#,
    )
    .bind(&name)
    .bind(&path)
    .bind(&share_type)
    .bind(&server)
    .bind(&purpose)
    .bind(&status)
    .bind(&notes)
    .bind(id)
    .execute(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM network_share WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!(
            "Network share with id '{}' not found",
            id
        )));
    }

    Ok(())
}
