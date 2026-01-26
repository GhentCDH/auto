use sqlx::SqlitePool;

use crate::models::{
    Application, ApplicationWithRelations, ClientRelation, CreateApplication, DomainRelation,
    HostRelation, NetworkShareRelation, Note, PaginatedResponse, PaginationParams, PersonRelation,
    UpdateApplication, new_id,
};
use crate::{Error, Result};

pub async fn list(
    pool: &SqlitePool,
    params: &PaginationParams,
) -> Result<PaginatedResponse<Application>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;

    let (applications, total): (Vec<Application>, i64) = if let Some(search) = &params.search {
        let search_pattern = format!("%{}%", search);
        let apps = sqlx::query_as::<_, Application>(
            r#"
            SELECT id, name, description, repository_url, status, created_at, updated_at, created_by
            FROM application
            WHERE name LIKE ?1 OR description LIKE ?1
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
            "SELECT COUNT(*) FROM application WHERE name LIKE ?1 OR description LIKE ?1",
        )
        .bind(&search_pattern)
        .fetch_one(pool)
        .await?;

        (apps, count.0)
    } else {
        let apps = sqlx::query_as::<_, Application>(
            r#"
            SELECT id, name, description, repository_url, status, created_at, updated_at, created_by
            FROM application
            ORDER BY name ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM application")
            .fetch_one(pool)
            .await?;

        (apps, count.0)
    };

    Ok(PaginatedResponse::new(applications, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Application> {
    sqlx::query_as::<_, Application>(
        r#"
        SELECT id, name, description, repository_url, status, created_at, updated_at, created_by
        FROM application
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Application with id '{}' not found", id)))
}

pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<ApplicationWithRelations> {
    let application = get(pool, id).await?;

    let hosts = sqlx::query_as::<_, HostRelation>(
        r#"
        SELECT h.id, h.name, h.host_type, h.hostname, h.ip_address, h.status,
               ah.role, ah.notes as relation_notes
        FROM host h
        JOIN application_host ah ON h.id = ah.host_id
        WHERE ah.application_id = ?1
        ORDER BY h.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let domains = sqlx::query_as::<_, DomainRelation>(
        r#"
        SELECT d.id, d.name, d.registrar, d.expires_at, d.ssl_expires_at, d.status,
               ad.record_type, ad.target, ad.target_host_id, ad.is_primary, ad.notes as relation_notes
        FROM domain d
        JOIN application_domain ad ON d.id = ad.domain_id
        WHERE ad.application_id = ?1
        ORDER BY ad.is_primary DESC, d.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let people = sqlx::query_as::<_, PersonRelation>(
        r#"
        SELECT p.id, p.name, p.email, p.role, p.is_active,
               ap.contribution_type, ap.start_date, ap.end_date, ap.notes as relation_notes
        FROM person p
        JOIN application_person ap ON p.id = ap.person_id
        WHERE ap.application_id = ?1
        ORDER BY p.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let clients = sqlx::query_as::<_, ClientRelation>(
        r#"
        SELECT c.id, c.name, c.contact_name, c.contact_email, c.status,
               ac.relationship_type, ac.contract_ref, ac.notes as relation_notes
        FROM client c
        JOIN application_client ac ON c.id = ac.client_id
        WHERE ac.application_id = ?1
        ORDER BY c.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let network_shares = sqlx::query_as::<_, NetworkShareRelation>(
        r#"
        SELECT ns.id, ns.name, ns.path, ns.share_type, ns.server, ns.status,
               ans.usage, ans.mount_point, ans.permissions, ans.notes as relation_notes
        FROM network_share ns
        JOIN application_network_share ans ON ns.id = ans.network_share_id
        WHERE ans.application_id = ?1
        ORDER BY ns.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let notes = sqlx::query_as::<_, Note>(
        r#"
        SELECT id, entity_type, entity_id, title, content, note_type, url, is_pinned, created_at, updated_at, created_by
        FROM note
        WHERE entity_type = 'application' AND entity_id = ?1
        ORDER BY is_pinned DESC, created_at DESC
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(ApplicationWithRelations {
        application,
        hosts,
        domains,
        people,
        clients,
        network_shares,
        notes,
    })
}

pub async fn create(pool: &SqlitePool, input: CreateApplication) -> Result<Application> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO application (id, name, description, repository_url, status)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.repository_url)
    .bind(&input.status)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: UpdateApplication) -> Result<Application> {
    let existing = get(pool, id).await?;

    let name = input.name.unwrap_or(existing.name);
    let description = input.description.or(existing.description);
    let repository_url = input.repository_url.or(existing.repository_url);
    let status = input.status.unwrap_or(existing.status);

    sqlx::query(
        r#"
        UPDATE application
        SET name = ?1, description = ?2, repository_url = ?3, status = ?4, updated_at = datetime('now')
        WHERE id = ?5
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(&repository_url)
    .bind(&status)
    .bind(id)
    .execute(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM application WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!(
            "Application with id '{}' not found",
            id
        )));
    }

    Ok(())
}

// Relationship management

pub async fn link_host(
    pool: &SqlitePool,
    app_id: &str,
    host_id: &str,
    role: &str,
    notes: Option<&str>,
) -> Result<()> {
    // Verify both entities exist
    get(pool, app_id).await?;
    crate::service::host::get(pool, host_id).await?;

    sqlx::query(
        r#"
        INSERT INTO application_host (application_id, host_id, role, notes)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT (application_id, host_id) DO UPDATE SET role = ?3, notes = ?4
        "#,
    )
    .bind(app_id)
    .bind(host_id)
    .bind(role)
    .bind(notes)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unlink_host(pool: &SqlitePool, app_id: &str, host_id: &str) -> Result<()> {
    let result =
        sqlx::query("DELETE FROM application_host WHERE application_id = ?1 AND host_id = ?2")
            .bind(app_id)
            .bind(host_id)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound("Relationship not found".to_string()));
    }

    Ok(())
}

pub async fn link_domain(
    pool: &SqlitePool,
    app_id: &str,
    domain_id: &str,
    record_type: &str,
    target: Option<&str>,
    target_host_id: Option<&str>,
    is_primary: bool,
    notes: Option<&str>,
) -> Result<()> {
    get(pool, app_id).await?;
    crate::service::domain::get(pool, domain_id).await?;

    sqlx::query(
        r#"
        INSERT INTO application_domain (application_id, domain_id, record_type, target, is_primary, notes, target_host_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT (application_id, domain_id) DO UPDATE SET record_type = ?3, target = ?4, is_primary = ?5, notes = ?6, target_host_id = ?7
        "#,
    )
    .bind(app_id)
    .bind(domain_id)
    .bind(record_type)
    .bind(target)
    .bind(is_primary)
    .bind(notes)
    .bind(target_host_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unlink_domain(pool: &SqlitePool, app_id: &str, domain_id: &str) -> Result<()> {
    let result =
        sqlx::query("DELETE FROM application_domain WHERE application_id = ?1 AND domain_id = ?2")
            .bind(app_id)
            .bind(domain_id)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound("Relationship not found".to_string()));
    }

    Ok(())
}

pub async fn link_person(
    pool: &SqlitePool,
    app_id: &str,
    person_id: &str,
    contribution_type: &str,
    start_date: Option<&str>,
    end_date: Option<&str>,
    notes: Option<&str>,
) -> Result<()> {
    get(pool, app_id).await?;
    crate::service::person::get(pool, person_id).await?;

    sqlx::query(
        r#"
        INSERT INTO application_person (application_id, person_id, contribution_type, start_date, end_date, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT (application_id, person_id) DO UPDATE SET contribution_type = ?3, start_date = ?4, end_date = ?5, notes = ?6
        "#,
    )
    .bind(app_id)
    .bind(person_id)
    .bind(contribution_type)
    .bind(start_date)
    .bind(end_date)
    .bind(notes)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unlink_person(pool: &SqlitePool, app_id: &str, person_id: &str) -> Result<()> {
    let result =
        sqlx::query("DELETE FROM application_person WHERE application_id = ?1 AND person_id = ?2")
            .bind(app_id)
            .bind(person_id)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound("Relationship not found".to_string()));
    }

    Ok(())
}

pub async fn link_client(
    pool: &SqlitePool,
    app_id: &str,
    client_id: &str,
    relationship_type: &str,
    contract_ref: Option<&str>,
    notes: Option<&str>,
) -> Result<()> {
    get(pool, app_id).await?;
    crate::service::client::get(pool, client_id).await?;

    sqlx::query(
        r#"
        INSERT INTO application_client (application_id, client_id, relationship_type, contract_ref, notes)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT (application_id, client_id) DO UPDATE SET relationship_type = ?3, contract_ref = ?4, notes = ?5
        "#,
    )
    .bind(app_id)
    .bind(client_id)
    .bind(relationship_type)
    .bind(contract_ref)
    .bind(notes)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unlink_client(pool: &SqlitePool, app_id: &str, client_id: &str) -> Result<()> {
    let result =
        sqlx::query("DELETE FROM application_client WHERE application_id = ?1 AND client_id = ?2")
            .bind(app_id)
            .bind(client_id)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound("Relationship not found".to_string()));
    }

    Ok(())
}

pub async fn link_network_share(
    pool: &SqlitePool,
    app_id: &str,
    share_id: &str,
    usage: Option<&str>,
    mount_point: Option<&str>,
    permissions: Option<&str>,
    notes: Option<&str>,
) -> Result<()> {
    get(pool, app_id).await?;
    crate::service::network_share::get(pool, share_id).await?;

    sqlx::query(
        r#"
        INSERT INTO application_network_share (application_id, network_share_id, usage, mount_point, permissions, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT (application_id, network_share_id) DO UPDATE SET usage = ?3, mount_point = ?4, permissions = ?5, notes = ?6
        "#,
    )
    .bind(app_id)
    .bind(share_id)
    .bind(usage)
    .bind(mount_point)
    .bind(permissions)
    .bind(notes)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unlink_network_share(pool: &SqlitePool, app_id: &str, share_id: &str) -> Result<()> {
    let result = sqlx::query(
        "DELETE FROM application_network_share WHERE application_id = ?1 AND network_share_id = ?2",
    )
    .bind(app_id)
    .bind(share_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound("Relationship not found".to_string()));
    }

    Ok(())
}
