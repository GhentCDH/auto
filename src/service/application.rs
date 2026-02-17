use sqlx::SqlitePool;
use tracing::info;

use crate::models::{
    Application, ApplicationWithRelations, CreateApplication, DomainRelation, InfraRelation,
    NetworkShareRelation, Note, PaginatedResponse, PaginationParams, PersonRelation,
    ServiceRelation, StackRelation, UpdateApplication, new_id,
};
use crate::{Error, Result, service};

pub async fn list(
    pool: &SqlitePool,
    params: &PaginationParams,
    status: Option<&str>,
    environment: Option<&str>,
) -> Result<PaginatedResponse<Application>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;
    let search_pattern = params.search.as_ref().map(|s| format!("%{}%", s));

    let applications = sqlx::query_as::<_, Application>(
        r#"
        SELECT id, name, description, repository_url, environment, url, status, created_at, updated_at, created_by
        FROM application
        WHERE (?1 IS NULL OR name LIKE ?1 OR description LIKE ?1)
          AND (?2 IS NULL OR status = ?2)
          AND (?3 IS NULL OR environment = ?3)
        ORDER BY name ASC
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
        FROM application
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

    Ok(PaginatedResponse::new(applications, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Application> {
    sqlx::query_as::<_, Application>(
        r#"
        SELECT id, name, description, repository_url, environment, url, status, created_at, updated_at, created_by
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

    let infra = sqlx::query_as::<_, InfraRelation>(
        r#"
        SELECT i.id, i.name, i.type, ai.notes as relation_notes
        FROM infra i
        JOIN application_infra ai ON i.id = ai.infra_id
        WHERE ai.application_id = ?1
        ORDER BY i.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let services = sqlx::query_as::<_, ServiceRelation>(
        r#"
        SELECT s.id, s.name, s.environment, s.status, asvc.notes as relation_notes
        FROM service s
        JOIN application_service asvc ON s.id = asvc.service_id
        WHERE asvc.application_id = ?1
        ORDER BY s.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    info!("Getting domains relation");

    let domains = sqlx::query_as::<_, DomainRelation>(
        r#"
        SELECT d.id, d.fqdn, d.target_application_id, ap.name as target_application_name, 
            d.target_service_id, s.name as target_service_name, 
            ad.notes as relation_notes
        FROM domain d
        JOIN application_domain ad ON d.id = ad.domain_id
        LEFT JOIN application ap ON d.target_application_id = ap.id
        LEFT JOIN service s ON d.target_service_id = s.id
        WHERE ad.application_id = ?1
        ORDER BY d.fqdn
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    info!("Getting peoples relation");

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

    let stacks = sqlx::query_as::<_, StackRelation>(
        r#"
        SELECT s.id, s.name
        FROM stack s
        JOIN application_stack ast ON s.id = ast.stack_id
        WHERE ast.application_id = ?1
        ORDER BY s.name
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let healthchecks = service::healthcheck::get_for_application(pool, id).await?;

    Ok(ApplicationWithRelations {
        application,
        infra,
        services,
        domains,
        people,
        network_shares,
        notes,
        stacks,
        healthchecks,
    })
}

pub async fn create(pool: &SqlitePool, input: CreateApplication) -> Result<Application> {
    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO application (id, name, description, repository_url, environment, url, status)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.repository_url)
    .bind(&input.environment)
    .bind(&input.url)
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
    let environment = input.environment.unwrap_or(existing.environment);
    let url = input.url.or(existing.url);
    let status = input.status.unwrap_or(existing.status);

    sqlx::query(
        r#"
        UPDATE application
        SET name = ?1, description = ?2, repository_url = ?3, environment = ?4, url = ?5, status = ?6, updated_at = datetime('now')
        WHERE id = ?7
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(&repository_url)
    .bind(&environment)
    .bind(&url)
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

pub async fn link_infra(
    pool: &SqlitePool,
    app_id: &str,
    infra_id: &str,
    notes: Option<&str>,
) -> Result<()> {
    // Verify both entities exist
    get(pool, app_id).await?;
    crate::service::infra::get(pool, infra_id).await?;

    sqlx::query(
        r#"
        INSERT INTO application_infra (application_id, infra_id, notes)
        VALUES (?1, ?2, ?3)
        ON CONFLICT (application_id, infra_id) DO UPDATE SET notes = ?3, updated_at = datetime('now')
        "#,
    )
    .bind(app_id)
    .bind(infra_id)
    .bind(notes)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unlink_infra(pool: &SqlitePool, app_id: &str, infra_id: &str) -> Result<()> {
    let result =
        sqlx::query("DELETE FROM application_infra WHERE application_id = ?1 AND infra_id = ?2")
            .bind(app_id)
            .bind(infra_id)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound("Relationship not found".to_string()));
    }

    Ok(())
}

pub async fn link_service(
    pool: &SqlitePool,
    app_id: &str,
    service_id: &str,
    notes: Option<&str>,
) -> Result<()> {
    // Verify both entities exist
    get(pool, app_id).await?;
    crate::service::service::get(pool, service_id).await?;

    sqlx::query(
        r#"
        INSERT INTO application_service (application_id, service_id, notes)
        VALUES (?1, ?2, ?3)
        ON CONFLICT (application_id, service_id) DO UPDATE SET notes = ?3, updated_at = datetime('now')
        "#,
    )
    .bind(app_id)
    .bind(service_id)
    .bind(notes)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unlink_service(pool: &SqlitePool, app_id: &str, service_id: &str) -> Result<()> {
    let result = sqlx::query(
        "DELETE FROM application_service WHERE application_id = ?1 AND service_id = ?2",
    )
    .bind(app_id)
    .bind(service_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound("Relationship not found".to_string()));
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn link_domain(
    pool: &SqlitePool,
    app_id: &str,
    domain_id: &str,
    notes: Option<&str>,
) -> Result<()> {
    let app_relations = get_with_relations(pool, app_id).await?;

    let domain = crate::service::domain::get(pool, domain_id).await?;

    // If this application isn't linked to the service target of the domain, link them
    if let Some(id) = domain.target_service_id
        && !app_relations.services.iter().any(|s| s.id == id)
    {
        link_service(
            pool,
            app_id,
            &id,
            Some(&format!("through '{}'", domain.fqdn)),
        )
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO application_domain (application_id, domain_id, notes)
        VALUES (?1, ?2, ?3)
        ON CONFLICT (application_id, domain_id) DO UPDATE SET notes = ?3
        "#,
    )
    .bind(app_id)
    .bind(domain_id)
    .bind(notes)
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

pub async fn link_stack(pool: &SqlitePool, app_id: &str, stack_id: &str) -> Result<()> {
    get(pool, app_id).await?;
    crate::service::stack::get(pool, stack_id).await?;

    sqlx::query(
        r#"
        INSERT INTO application_stack (application_id, stack_id)
        VALUES (?1, ?2)
        ON CONFLICT (application_id, stack_id) DO NOTHING
        "#,
    )
    .bind(app_id)
    .bind(stack_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unlink_stack(pool: &SqlitePool, app_id: &str, stack_id: &str) -> Result<()> {
    let result =
        sqlx::query("DELETE FROM application_stack WHERE application_id = ?1 AND stack_id = ?2")
            .bind(app_id)
            .bind(stack_id)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound("Relationship not found".to_string()));
    }

    Ok(())
}
