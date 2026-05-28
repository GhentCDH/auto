use sqlx::SqlitePool;

use crate::models::{
    ApplicationInfraRelation, CreateDomain, CreateInfra, Infra, InfraDomainRef, InfraIp,
    InfraListItem, InfraWithRelations, NewInfraDomain, PaginatedResponse, PaginationParams,
    ServiceInfraRelation, UpdateDomain, UpdateInfra, new_id,
};
use crate::{Error, Result, service};

pub async fn list(
    pool: &SqlitePool,
    params: &PaginationParams,
    infra_type: Option<&str>,
    ip: Option<&str>,
) -> Result<PaginatedResponse<InfraListItem>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;
    let search_pattern = params.search.as_ref().map(|s| format!("%{}%", s));
    let ip_pattern = ip.map(|s| format!("%{}%", s));

    // EXISTS subquery keeps each infra row distinct even when multiple IPs match.
    let items = sqlx::query_as::<_, Infra>(
        r#"
        SELECT id, name, description, type, created_at, updated_at, created_by
        FROM infra
        WHERE (?1 IS NULL OR name LIKE ?1 OR description LIKE ?1)
          AND (?2 IS NULL OR type = ?2)
          AND (?3 IS NULL OR EXISTS (
              SELECT 1 FROM infra_ip WHERE infra_id = infra.id AND ip LIKE ?3
          ))
        ORDER BY name COLLATE NOCASE ASC
        LIMIT ?4 OFFSET ?5
        "#,
    )
    .bind(&search_pattern)
    .bind(infra_type)
    .bind(&ip_pattern)
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
          AND (?3 IS NULL OR EXISTS (
              SELECT 1 FROM infra_ip WHERE infra_id = infra.id AND ip LIKE ?3
          ))
        "#,
    )
    .bind(&search_pattern)
    .bind(infra_type)
    .bind(&ip_pattern)
    .fetch_one(pool)
    .await?;

    let list_items = attach_ips(pool, items).await?;
    Ok(PaginatedResponse::new(list_items, total, params))
}

/// Batch-fetch IPs for the given infras and return list rows. One query for the
/// whole page beats N round-trips; an empty page short-circuits with no query.
async fn attach_ips(pool: &SqlitePool, items: Vec<Infra>) -> Result<Vec<InfraListItem>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    let mut qb = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT infra_id, ip, source, last_synced_at FROM infra_ip WHERE infra_id IN (",
    );
    let mut sep = qb.separated(", ");
    for item in &items {
        sep.push_bind(item.id.clone());
    }
    qb.push(") ORDER BY source, ip");

    let rows: Vec<(String, String, String, String)> = qb.build_query_as().fetch_all(pool).await?;

    let mut by_id: std::collections::HashMap<String, Vec<InfraIp>> =
        std::collections::HashMap::new();
    for (infra_id, ip, source, last_synced_at) in rows {
        by_id.entry(infra_id).or_default().push(InfraIp {
            ip,
            source,
            last_synced_at,
        });
    }

    Ok(items
        .into_iter()
        .map(|infra| {
            let ips = by_id.remove(&infra.id).unwrap_or_default();
            InfraListItem { infra, ips }
        })
        .collect())
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

    let ips = sqlx::query_as::<_, InfraIp>(
        r#"
        SELECT ip, source, last_synced_at
        FROM infra_ip
        WHERE infra_id = ?1
        ORDER BY source, ip
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let domain = sqlx::query_as::<_, InfraDomainRef>(
        r#"
        SELECT id, fqdn FROM domain WHERE target_infra_id = ?1 LIMIT 1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

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
        ips,
        domain,
        applications,
        services,
    })
}

/// Point a domain at this infra (sets `domain.target_infra_id`, clearing any
/// app/service target per the XOR). This is the cross-entity write behind the
/// infra form's "Domain" option — single source of truth stays on the domain.
async fn set_domain_target(pool: &SqlitePool, domain_id: &str, infra_id: &str) -> Result<()> {
    service::domain::update(
        pool,
        domain_id,
        UpdateDomain {
            fqdn: None,
            registrar: None,
            dns_provider: None,
            expires_at: None,
            notes: None,
            target_application_id: None,
            target_service_id: None,
            target_infra_id: Some(infra_id.to_string()),
        },
    )
    .await?;
    Ok(())
}

/// Create a fresh domain that targets this infra. Breaks the circular create
/// order — the new domain gets its required target (`target_infra_id`) from the
/// infra that was just inserted, instead of needing one to exist beforehand.
async fn create_domain_target(
    pool: &SqlitePool,
    new_domain: NewInfraDomain,
    infra_id: &str,
) -> Result<()> {
    service::domain::create(
        pool,
        CreateDomain {
            fqdn: new_domain.fqdn,
            registrar: new_domain.registrar,
            dns_provider: new_domain.dns_provider,
            expires_at: new_domain.expires_at,
            notes: new_domain.notes,
            target_application_id: None,
            target_service_id: None,
            target_infra_id: Some(infra_id.to_string()),
        },
    )
    .await?;
    Ok(())
}

/// Replace the manually-assigned IPs for an infra. Only touches `source='manual'`
/// rows — domain-resolved IPs are left untouched.
async fn replace_manual_ips(pool: &SqlitePool, infra_id: &str, ips: &[String]) -> Result<()> {
    sqlx::query("DELETE FROM infra_ip WHERE infra_id = ?1 AND source = 'manual'")
        .bind(infra_id)
        .execute(pool)
        .await?;
    for ip in ips {
        let ip = ip.trim();
        if ip.is_empty() {
            continue;
        }
        sqlx::query(
            r#"
            INSERT INTO infra_ip (infra_id, ip, source, last_synced_at)
            VALUES (?1, ?2, 'manual', datetime('now'))
            ON CONFLICT (infra_id, ip) DO UPDATE SET source = 'manual', last_synced_at = datetime('now')
            "#,
        )
        .bind(infra_id)
        .bind(ip)
        .execute(pool)
        .await?;
    }
    Ok(())
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

    if let Some(domain_id) = &input.domain_id {
        set_domain_target(pool, domain_id, &id).await?;
    }
    if let Some(new_domain) = input.new_domain {
        create_domain_target(pool, new_domain, &id).await?;
    }
    if let Some(manual_ips) = &input.manual_ips {
        replace_manual_ips(pool, &id, manual_ips).await?;
    }

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

    if let Some(domain_id) = &input.domain_id {
        set_domain_target(pool, domain_id, id).await?;
    }
    if let Some(new_domain) = input.new_domain {
        create_domain_target(pool, new_domain, id).await?;
    }
    if let Some(manual_ips) = &input.manual_ips {
        replace_manual_ips(pool, id, manual_ips).await?;
    }

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
