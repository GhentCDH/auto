use sqlx::SqlitePool;

use crate::models::{
    new_id, ApplicationClientRelation, Client, ClientWithRelations, CreateClient, PaginatedResponse,
    PaginationParams, UpdateClient,
};
use crate::{Error, Result};

pub async fn list(pool: &SqlitePool, params: &PaginationParams) -> Result<PaginatedResponse<Client>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;

    let (clients, total): (Vec<Client>, i64) = if let Some(search) = &params.search {
        let search_pattern = format!("%{}%", search);
        let items = sqlx::query_as::<_, Client>(
            r#"
            SELECT id, name, contact_name, contact_email, department, phone, address, status, notes, created_at, updated_at, created_by
            FROM client
            WHERE name LIKE ?1 OR contact_name LIKE ?1 OR contact_email LIKE ?1
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
            "SELECT COUNT(*) FROM client WHERE name LIKE ?1 OR contact_name LIKE ?1 OR contact_email LIKE ?1",
        )
        .bind(&search_pattern)
        .fetch_one(pool)
        .await?;

        (items, count.0)
    } else {
        let items = sqlx::query_as::<_, Client>(
            r#"
            SELECT id, name, contact_name, contact_email, department, phone, address, status, notes, created_at, updated_at, created_by
            FROM client
            ORDER BY name ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM client")
            .fetch_one(pool)
            .await?;

        (items, count.0)
    };

    Ok(PaginatedResponse::new(clients, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Client> {
    sqlx::query_as::<_, Client>(
        r#"
        SELECT id, name, contact_name, contact_email, department, phone, address, status, notes, created_at, updated_at, created_by
        FROM client
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Client with id '{}' not found", id)))
}

pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<ClientWithRelations> {
    let client = get(pool, id).await?;

    let applications = sqlx::query_as::<_, ApplicationClientRelation>(
        r#"
        SELECT a.id, a.name, a.status, ac.relationship_type
        FROM application a
        JOIN application_client ac ON a.id = ac.application_id
        WHERE ac.client_id = ?1
        ORDER BY a.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(ClientWithRelations { client, applications })
}

pub async fn create(pool: &SqlitePool, input: CreateClient) -> Result<Client> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO client (id, name, contact_name, contact_email, department, phone, address, status, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.contact_name)
    .bind(&input.contact_email)
    .bind(&input.department)
    .bind(&input.phone)
    .bind(&input.address)
    .bind(&input.status)
    .bind(&input.notes)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: UpdateClient) -> Result<Client> {
    let existing = get(pool, id).await?;

    let name = input.name.unwrap_or(existing.name);
    let contact_name = input.contact_name.or(existing.contact_name);
    let contact_email = input.contact_email.or(existing.contact_email);
    let department = input.department.or(existing.department);
    let phone = input.phone.or(existing.phone);
    let address = input.address.or(existing.address);
    let status = input.status.unwrap_or(existing.status);
    let notes = input.notes.or(existing.notes);

    sqlx::query(
        r#"
        UPDATE client
        SET name = ?1, contact_name = ?2, contact_email = ?3, department = ?4, phone = ?5, address = ?6, status = ?7, notes = ?8, updated_at = datetime('now')
        WHERE id = ?9
        "#,
    )
    .bind(&name)
    .bind(&contact_name)
    .bind(&contact_email)
    .bind(&department)
    .bind(&phone)
    .bind(&address)
    .bind(&status)
    .bind(&notes)
    .bind(id)
    .execute(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM client WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!("Client with id '{}' not found", id)));
    }

    Ok(())
}
