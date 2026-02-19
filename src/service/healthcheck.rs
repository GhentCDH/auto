use std::collections::HashMap;
use std::time::Instant;

use reqwest::Client;
use sqlx::SqlitePool;

use crate::models::{
    CreateHealthcheck, Healthcheck, HealthcheckExecuteResult, HealthcheckRelation,
    HealthcheckWithRelations, KumaMonitor, PaginatedResponse, PaginationParams, UpdateHealthcheck,
    new_id,
};
use crate::{Error, Result};

pub async fn list(
    pool: &SqlitePool,
    params: &PaginationParams,
    application_id: Option<&str>,
    service_id: Option<&str>,
    is_enabled: Option<bool>,
) -> Result<PaginatedResponse<HealthcheckWithRelations>> {
    let limit = params.limit() as i32;
    let offset = params.offset() as i32;
    let search_pattern = params.search.as_ref().map(|s| format!("%{}%", s));

    let healthchecks = sqlx::query_as::<_, Healthcheck>(
        r#"
        SELECT h.id, h.name, h.application_id, h.service_id, h.domain_id,
               h.protocol, h.path, h.method, h.headers, h.expected_status,
               h.expected_body, h.timeout_seconds, h.is_enabled, h.notes,
               h.retry, h.retry_interval, h.request_body_encoding, h.request_body,
               h.http_auth_user, h.http_auth_pass, h.kuma_id,
               h.created_at, h.updated_at, h.created_by
        FROM healthcheck h
        WHERE (?1 IS NULL OR h.name LIKE ?1)
          AND (?2 IS NULL OR h.application_id = ?2)
          AND (?3 IS NULL OR h.service_id = ?3)
          AND (?4 IS NULL OR h.is_enabled = ?4)
        ORDER BY h.name ASC
        LIMIT ?5 OFFSET ?6
        "#,
    )
    .bind(&search_pattern)
    .bind(application_id)
    .bind(service_id)
    .bind(is_enabled)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(healthchecks.len());
    for hc in healthchecks {
        result.push(extend_relations(pool, hc).await?);
    }

    let (total,) = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*)
        FROM healthcheck h
        WHERE (?1 IS NULL OR h.name LIKE ?1)
          AND (?2 IS NULL OR h.application_id = ?2)
          AND (?3 IS NULL OR h.service_id = ?3)
          AND (?4 IS NULL OR h.is_enabled = ?4)
        "#,
    )
    .bind(&search_pattern)
    .bind(application_id)
    .bind(service_id)
    .bind(is_enabled)
    .fetch_one(pool)
    .await?;

    Ok(PaginatedResponse::new(result, total, params))
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Healthcheck> {
    sqlx::query_as::<_, Healthcheck>(
        r#"
        SELECT id, name, application_id, service_id, domain_id,
               protocol, path, method, headers, expected_status,
               expected_body, timeout_seconds, is_enabled, notes,
               retry, retry_interval, request_body_encoding, request_body,
               http_auth_user, http_auth_pass, kuma_id,
               created_at, updated_at, created_by
        FROM healthcheck
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::NotFound(format!("Healthcheck with id '{}' not found", id)))
}

async fn extend_relations(
    pool: &SqlitePool,
    healthcheck: Healthcheck,
) -> Result<HealthcheckWithRelations> {
    // Get application name if linked
    let application_name = if let Some(app_id) = &healthcheck.application_id {
        sqlx::query_scalar::<_, String>("SELECT name FROM application WHERE id = ?1")
            .bind(app_id)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    // Get service name if linked
    let service_name = if let Some(svc_id) = &healthcheck.service_id {
        sqlx::query_scalar::<_, String>("SELECT name FROM service WHERE id = ?1")
            .bind(svc_id)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    // Get domain FQDN
    let domain_fqdn = sqlx::query_scalar::<_, String>("SELECT fqdn FROM domain WHERE id = ?1")
        .bind(&healthcheck.domain_id)
        .fetch_one(pool)
        .await?;

    // Parse headers JSON
    let parsed_headers = healthcheck
        .headers
        .as_ref()
        .and_then(|h| serde_json::from_str::<HashMap<String, String>>(h).ok());

    Ok(HealthcheckWithRelations {
        healthcheck,
        application_name,
        service_name,
        domain_fqdn,
        parsed_headers,
    })
}

pub async fn get_with_relations(pool: &SqlitePool, id: &str) -> Result<HealthcheckWithRelations> {
    let healthcheck = get(pool, id).await?;
    extend_relations(pool, healthcheck).await
}

pub async fn create(pool: &SqlitePool, input: CreateHealthcheck) -> Result<Healthcheck> {
    // Validate XOR constraint
    match (&input.application_id, &input.service_id) {
        (Some(_), Some(_)) => {
            return Err(Error::ValidationError(
                "Cannot set both application_id and service_id".into(),
            ));
        }
        (None, None) => {
            return Err(Error::ValidationError(
                "Must set either application_id or service_id".into(),
            ));
        }
        _ => {}
    }

    // Validate target exists
    if let Some(app_id) = &input.application_id {
        crate::service::application::get(pool, app_id).await?;
    }
    if let Some(svc_id) = &input.service_id {
        crate::service::service::get(pool, svc_id).await?;
    }

    // Validate domain exists
    crate::service::domain::get(pool, &input.domain_id).await?;

    // Validate headers is valid JSON if provided
    if let Some(headers) = &input.headers {
        serde_json::from_str::<HashMap<String, String>>(headers).map_err(|_| {
            Error::ValidationError("headers must be valid JSON object with string values".into())
        })?;
    }

    let id = new_id();

    sqlx::query(
        r#"
        INSERT INTO healthcheck (id, name, application_id, service_id, kuma_id, domain_id,
                                 protocol, path, method, headers, expected_status,
                                 expected_body, timeout_seconds, is_enabled, notes,
                                 retry, retry_interval, request_body_encoding, request_body,
                                 http_auth_user, http_auth_pass)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)
        "#,
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.application_id)
    .bind(&input.service_id)
    .bind(input.kuma_id)
    .bind(&input.domain_id)
    .bind(&input.protocol)
    .bind(&input.path)
    .bind(&input.method)
    .bind(&input.headers)
    .bind(input.expected_status)
    .bind(&input.expected_body)
    .bind(input.timeout_seconds)
    .bind(input.is_enabled)
    .bind(&input.notes)
    .bind(input.retry)
    .bind(input.retry_interval)
    .bind(&input.request_body_encoding)
    .bind(&input.request_body)
    .bind(&input.http_auth_user)
    .bind(&input.http_auth_pass)
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: UpdateHealthcheck) -> Result<Healthcheck> {
    let existing = get(pool, id).await?;

    // Determine new target values
    let (application_id, service_id) = match (&input.application_id, &input.service_id) {
        (Some(app_id), None) => (Some(app_id.clone()), None),
        (None, Some(svc_id)) => (None, Some(svc_id.clone())),
        (Some(_), Some(_)) => {
            return Err(Error::ValidationError(
                "Cannot set both application_id and service_id".into(),
            ));
        }
        (None, None) => (existing.application_id, existing.service_id),
    };

    // Validate target exists if changed
    if let Some(app_id) = &application_id {
        crate::service::application::get(pool, app_id).await?;
    }
    if let Some(svc_id) = &service_id {
        crate::service::service::get(pool, svc_id).await?;
    }

    // Validate domain exists if changed
    let domain_id = input.domain_id.unwrap_or(existing.domain_id);
    crate::service::domain::get(pool, &domain_id).await?;

    // Validate headers is valid JSON if provided
    let headers = input.headers.or(existing.headers);
    if let Some(h) = &headers {
        serde_json::from_str::<HashMap<String, String>>(h).map_err(|_| {
            Error::ValidationError("headers must be valid JSON object with string values".into())
        })?;
    }

    let name = input.name.unwrap_or(existing.name);
    let protocol = input.protocol.unwrap_or(existing.protocol);
    let path = input.path.unwrap_or(existing.path);
    let method = input.method.unwrap_or(existing.method);
    let expected_status = input.expected_status.unwrap_or(existing.expected_status);
    let expected_body = input.expected_body.or(existing.expected_body);
    let timeout_seconds = input.timeout_seconds.unwrap_or(existing.timeout_seconds);
    let is_enabled = input.is_enabled.unwrap_or(existing.is_enabled);
    let notes = input.notes.or(existing.notes);
    let retry = input.retry.unwrap_or(existing.retry);
    let retry_interval = input.retry_interval.unwrap_or(existing.retry_interval);
    let request_body_encoding = input
        .request_body_encoding
        .unwrap_or(existing.request_body_encoding);
    let request_body = input.request_body.or(existing.request_body);
    let http_auth_user = input.http_auth_user.or(existing.http_auth_user);
    let http_auth_pass = input.http_auth_pass.or(existing.http_auth_pass);
    let kuma_id = input.kuma_id.or(existing.kuma_id);

    sqlx::query(
        r#"
        UPDATE healthcheck
        SET name = ?1, application_id = ?2, service_id = ?3, domain_id = ?4,
            protocol = ?5, path = ?6, method = ?7, headers = ?8,
            expected_status = ?9, expected_body = ?10, timeout_seconds = ?11,
            is_enabled = ?12, notes = ?13, retry = ?14, retry_interval = ?15,
            request_body_encoding = ?16, request_body = ?17,
            http_auth_user = ?18, http_auth_pass = ?19, kuma_id = ?21
            updated_at = datetime('now')
        WHERE id = ?20
        "#,
    )
    .bind(&name)
    .bind(&application_id)
    .bind(&service_id)
    .bind(&domain_id)
    .bind(&protocol)
    .bind(&path)
    .bind(&method)
    .bind(&headers)
    .bind(expected_status)
    .bind(&expected_body)
    .bind(timeout_seconds)
    .bind(is_enabled)
    .bind(&notes)
    .bind(retry)
    .bind(retry_interval)
    .bind(&request_body_encoding)
    .bind(&request_body)
    .bind(&http_auth_user)
    .bind(&http_auth_pass)
    .bind(id)
    .bind(kuma_id)
    .execute(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM healthcheck WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!(
            "Healthcheck with id '{}' not found",
            id
        )));
    }

    Ok(())
}

/// Execute a healthcheck and return the result
pub async fn execute(pool: &SqlitePool, id: &str) -> Result<HealthcheckExecuteResult> {
    let healthcheck = get_with_relations(pool, id).await?;

    // Build URL
    let url = format!(
        "{}://{}{}",
        healthcheck.healthcheck.protocol, healthcheck.domain_fqdn, healthcheck.healthcheck.path
    );

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(
            healthcheck.healthcheck.timeout_seconds as u64,
        ))
        .danger_accept_invalid_certs(true) // For internal services with self-signed certs
        .build()
        .map_err(|e| Error::InternalError(format!("Failed to create HTTP client: {}", e)))?;

    // Validate HTTP method upfront
    if !["GET", "HEAD", "POST", "PUT", "PATCH", "DELETE"]
        .contains(&healthcheck.healthcheck.method.as_str())
    {
        return Err(Error::ValidationError(format!(
            "Unsupported HTTP method: {}",
            healthcheck.healthcheck.method
        )));
    }

    // Helper to build a fresh request
    let build_request = || {
        let mut req = match healthcheck.healthcheck.method.as_str() {
            "GET" => client.get(&url),
            "HEAD" => client.head(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "PATCH" => client.patch(&url),
            "DELETE" => client.delete(&url),
            _ => unreachable!(),
        };

        // Add HTTP Basic Authentication if configured
        if let (Some(user), Some(pass)) = (
            &healthcheck.healthcheck.http_auth_user,
            &healthcheck.healthcheck.http_auth_pass,
        ) {
            req = req.basic_auth(user, Some(pass));
        }

        // Add custom headers
        if let Some(headers) = &healthcheck.parsed_headers {
            for (key, value) in headers {
                req = req.header(key, value);
            }
        }

        // Add request body if provided (for POST/PUT/PATCH)
        if let Some(body) = &healthcheck.healthcheck.request_body {
            let content_type = match healthcheck.healthcheck.request_body_encoding.as_str() {
                "JSON" => "application/json",
                "x-www-form-urlencoded" => "application/x-www-form-urlencoded",
                "XML" => "application/xml",
                _ => "application/json",
            };
            req = req.header("Content-Type", content_type).body(body.clone());
        }

        req
    };

    let start = Instant::now();
    let executed_at = chrono::Utc::now().to_rfc3339();

    // Execute with retry support
    let max_attempts = healthcheck.healthcheck.retry + 1;
    let retry_interval_ms = healthcheck.healthcheck.retry_interval as u64 * 1000;
    let mut last_result = None;

    for attempt in 0..max_attempts {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(retry_interval_ms)).await;
        }

        let request = build_request();
        last_result = Some(request.send().await);

        // Check if successful to potentially skip remaining retries
        if let Some(Ok(ref response)) = last_result
            && response.status().as_u16() == healthcheck.healthcheck.expected_status as u16
        {
            break;
        }
    }

    let result = last_result.unwrap();
    let response_time_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let status_ok = status_code == healthcheck.healthcheck.expected_status as u16;

            // Check body if expected_body is set
            let body_match = if healthcheck.healthcheck.expected_body.is_some() {
                match response.text().await {
                    Ok(body) => Some(
                        body.contains(
                            healthcheck
                                .healthcheck
                                .expected_body
                                .as_ref()
                                .unwrap_or(&String::new()),
                        ),
                    ),
                    Err(_) => Some(false),
                }
            } else {
                None
            };

            let success = status_ok && body_match.unwrap_or(true);

            Ok(HealthcheckExecuteResult {
                healthcheck_id: id.to_string(),
                url,
                success,
                status_code: Some(status_code),
                response_time_ms,
                body_match,
                error: None,
                executed_at,
            })
        }
        Err(e) => Ok(HealthcheckExecuteResult {
            healthcheck_id: id.to_string(),
            url,
            success: false,
            status_code: None,
            response_time_ms,
            body_match: None,
            error: Some(e.to_string()),
            executed_at,
        }),
    }
}

/// Export all enabled healthchecks in Uptime Kuma format
pub async fn export_kuma(pool: &SqlitePool) -> Result<Vec<KumaMonitor>> {
    let healthchecks = sqlx::query_as::<_, Healthcheck>(
        r#"
        SELECT id, name, application_id, service_id, domain_id,
               protocol, path, method, headers, expected_status,
               expected_body, timeout_seconds, is_enabled, notes,
               retry, retry_interval, request_body_encoding, request_body,
               http_auth_user, http_auth_pass,
               created_at, updated_at, created_by
        FROM healthcheck
        WHERE is_enabled = 1
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut monitors = Vec::with_capacity(healthchecks.len());

    for hc in healthchecks {
        let with_relations = extend_relations(pool, hc).await?;

        let (target_type, target_name) = if with_relations.application_name.is_some() {
            (
                "application".to_string(),
                with_relations.application_name.unwrap_or_default(),
            )
        } else {
            (
                "service".to_string(),
                with_relations.service_name.unwrap_or_default(),
            )
        };

        let url = format!(
            "{}://{}{}",
            with_relations.healthcheck.protocol,
            with_relations.domain_fqdn,
            with_relations.healthcheck.path
        );

        monitors.push(KumaMonitor {
            name: with_relations.healthcheck.name,
            url,
            method: with_relations.healthcheck.method,
            expected_status: with_relations.healthcheck.expected_status,
            timeout: with_relations.healthcheck.timeout_seconds,
            headers: with_relations.parsed_headers,
            target_type,
            target_name,
        });
    }

    Ok(monitors)
}

/// Get healthcheck relations for an application
pub async fn get_for_application(
    pool: &SqlitePool,
    app_id: &str,
) -> Result<Vec<HealthcheckRelation>> {
    sqlx::query_as::<_, HealthcheckRelation>(
        r#"
        SELECT h.id, h.name, h.protocol, h.kuma_id, d.fqdn as domain_fqdn, 
               h.path, h.expected_status, h.is_enabled
        FROM healthcheck h
        JOIN domain d ON h.domain_id = d.id
        WHERE h.application_id = ?1
        ORDER BY h.name
        "#,
    )
    .bind(app_id)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Get healthcheck relations for a service
pub async fn get_for_service(
    pool: &SqlitePool,
    service_id: &str,
) -> Result<Vec<HealthcheckRelation>> {
    sqlx::query_as::<_, HealthcheckRelation>(
        r#"
        SELECT h.id, h.name, h.protocol, d.fqdn as domain_fqdn, 
               h.path, h.expected_status, h.is_enabled, h.kuma_id
        FROM healthcheck h
        JOIN domain d ON h.domain_id = d.id
        WHERE h.service_id = ?1
        ORDER BY h.name
        "#,
    )
    .bind(service_id)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}
