use sqlx::SqlitePool;

use crate::models::{
    ApplicationPersonRelation, CreatePerson, PaginatedResponse, PaginationParams, Person,
    PersonWithRelations, UpdatePerson, new_id,
};
use crate::{Error, Result};

pub async fn list(
    pool: &SqlitePool,
    params: &PaginationParams,
) -> Result<PaginatedResponse<Person>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;

    let (people, total): (Vec<Person>, i64) = if let Some(search) = &params.search {
        let search_pattern = format!("%{}%", search);
        let items = sqlx::query_as::<_, Person>(
            r#"
            SELECT id, name, email, role, department, phone, is_active, notes, created_at, updated_at, created_by
            FROM person
            WHERE name LIKE ?1 OR email LIKE ?1 OR role LIKE ?1
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
            "SELECT COUNT(*) FROM person WHERE name LIKE ?1 OR email LIKE ?1 OR role LIKE ?1",
        )
        .bind(&search_pattern)
        .fetch_one(pool)
        .await?;

        (items, count.0)
    } else {
        let items = sqlx::query_as::<_, Person>(
            r#"
            SELECT id, name, email, role, department, phone, is_active, notes, created_at, updated_at, created_by
            FROM person
            ORDER BY name ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM person")
            .fetch_one(pool)
            .await?;

        (items, count.0)
    };

    Ok(PaginatedResponse::new(people, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Person> {
    sqlx::query_as::<_, Person>(
        r#"
        SELECT id, name, email, role, department, phone, is_active, notes, created_at, updated_at, created_by
        FROM person
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Person with id '{}' not found", id)))
}

pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<PersonWithRelations> {
    let person = get(pool, id).await?;

    let applications = sqlx::query_as::<_, ApplicationPersonRelation>(
        r#"
        SELECT a.id, a.name, a.status, ap.contribution_type
        FROM application a
        JOIN application_person ap ON a.id = ap.application_id
        WHERE ap.person_id = ?1
        ORDER BY a.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(PersonWithRelations {
        person,
        applications,
    })
}

pub async fn create(pool: &SqlitePool, input: CreatePerson) -> Result<Person> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO person (id, name, email, role, department, phone, is_active, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.email)
    .bind(&input.role)
    .bind(&input.department)
    .bind(&input.phone)
    .bind(input.is_active)
    .bind(&input.notes)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: UpdatePerson) -> Result<Person> {
    let existing = get(pool, id).await?;

    let name = input.name.unwrap_or(existing.name);
    let email = input.email.or(existing.email);
    let role = input.role.or(existing.role);
    let department = input.department.or(existing.department);
    let phone = input.phone.or(existing.phone);
    let is_active = input.is_active.unwrap_or(existing.is_active);
    let notes = input.notes.or(existing.notes);

    sqlx::query(
        r#"
        UPDATE person
        SET name = ?1, email = ?2, role = ?3, department = ?4, phone = ?5, is_active = ?6, notes = ?7, updated_at = datetime('now')
        WHERE id = ?8
        "#,
    )
    .bind(&name)
    .bind(&email)
    .bind(&role)
    .bind(&department)
    .bind(&phone)
    .bind(is_active)
    .bind(&notes)
    .bind(id)
    .execute(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM person WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!(
            "Person with id '{}' not found",
            id
        )));
    }

    Ok(())
}
