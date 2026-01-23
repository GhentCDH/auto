use sqlx::SqlitePool;

use crate::models::{
    new_id, ApplicationHostRelation, CreateHost, Host, HostWithRelations, PaginatedResponse,
    PaginationParams, UpdateHost,
};
use crate::{Error, Result};

pub async fn list(pool: &SqlitePool, params: &PaginationParams) -> Result<PaginatedResponse<Host>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;

    let (hosts, total): (Vec<Host>, i64) = if let Some(search) = &params.search {
        let search_pattern = format!("%{}%", search);
        let items = sqlx::query_as::<_, Host>(
            r#"
            SELECT id, name, host_type, hostname, ip_address, location, os, specs, status, notes, created_at, updated_at, created_by
            FROM host
            WHERE name LIKE ?1 OR hostname LIKE ?1 OR ip_address LIKE ?1
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
            "SELECT COUNT(*) FROM host WHERE name LIKE ?1 OR hostname LIKE ?1 OR ip_address LIKE ?1",
        )
        .bind(&search_pattern)
        .fetch_one(pool)
        .await?;

        (items, count.0)
    } else {
        let items = sqlx::query_as::<_, Host>(
            r#"
            SELECT id, name, host_type, hostname, ip_address, location, os, specs, status, notes, created_at, updated_at, created_by
            FROM host
            ORDER BY name ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM host")
            .fetch_one(pool)
            .await?;

        (items, count.0)
    };

    Ok(PaginatedResponse::new(hosts, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Host> {
    sqlx::query_as::<_, Host>(
        r#"
        SELECT id, name, host_type, hostname, ip_address, location, os, specs, status, notes, created_at, updated_at, created_by
        FROM host
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Host with id '{}' not found", id)))
}

pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<HostWithRelations> {
    let host = get(pool, id).await?;

    let applications = sqlx::query_as::<_, ApplicationHostRelation>(
        r#"
        SELECT a.id, a.name, a.status, ah.role
        FROM application a
        JOIN application_host ah ON a.id = ah.application_id
        WHERE ah.host_id = ?1
        ORDER BY a.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(HostWithRelations { host, applications })
}

pub async fn create(pool: &SqlitePool, input: CreateHost) -> Result<Host> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO host (id, name, host_type, hostname, ip_address, location, os, specs, status, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.host_type)
    .bind(&input.hostname)
    .bind(&input.ip_address)
    .bind(&input.location)
    .bind(&input.os)
    .bind(&input.specs)
    .bind(&input.status)
    .bind(&input.notes)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: UpdateHost) -> Result<Host> {
    let existing = get(pool, id).await?;

    let name = input.name.unwrap_or(existing.name);
    let host_type = input.host_type.unwrap_or(existing.host_type);
    let hostname = input.hostname.or(existing.hostname);
    let ip_address = input.ip_address.or(existing.ip_address);
    let location = input.location.or(existing.location);
    let os = input.os.or(existing.os);
    let specs = input.specs.or(existing.specs);
    let status = input.status.unwrap_or(existing.status);
    let notes = input.notes.or(existing.notes);

    sqlx::query(
        r#"
        UPDATE host
        SET name = ?1, host_type = ?2, hostname = ?3, ip_address = ?4, location = ?5, os = ?6, specs = ?7, status = ?8, notes = ?9, updated_at = datetime('now')
        WHERE id = ?10
        "#,
    )
    .bind(&name)
    .bind(&host_type)
    .bind(&hostname)
    .bind(&ip_address)
    .bind(&location)
    .bind(&os)
    .bind(&specs)
    .bind(&status)
    .bind(&notes)
    .bind(id)
    .execute(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM host WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!("Host with id '{}' not found", id)));
    }

    Ok(())
}
