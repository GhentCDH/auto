use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use tracing::instrument;

use crate::models::{
    Application, ApplicationWithRelations, CreateApplication, LinkDomain, LinkInfra,
    LinkNetworkShare, LinkPerson, LinkService, PaginationParams, UpdateApplication,
};
use crate::service::application;
use crate::{AppState, Result};
use crate::{api::complete_id_of_type, overview::Overview as _};

#[derive(Debug, Deserialize, Default)]
pub struct ApplicationFilters {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub status: Option<String>,
    pub environment: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
        .route("/{id}/overview.md", get(get_overview_md))
        .route("/{id}/sync-outline", post(sync_outline))
        // Relationship management
        .route(
            "/{id}/infra/{infra_id}",
            post(link_infra).delete(unlink_infra),
        )
        .route(
            "/{id}/services/{service_id}",
            post(link_service).delete(unlink_service),
        )
        .route(
            "/{id}/domains/{domain_id}",
            post(link_domain).delete(unlink_domain),
        )
        .route(
            "/{id}/people/{person_id}",
            post(link_person).delete(unlink_person),
        )
        .route(
            "/{id}/shares/{share_id}",
            post(link_share).delete(unlink_share),
        )
        .route(
            "/{id}/stacks/{stack_id}",
            post(link_stack).delete(unlink_stack),
        )
}

#[utoipa::path(
    get,
    path = "/api/applications",
    tag = "applications",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("per_page" = Option<u32>, Query, description = "Items per page (max 100)"),
        ("search" = Option<String>, Query, description = "Search query"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("environment" = Option<String>, Query, description = "Filter by environment"),
    ),
    responses(
        (status = 200, description = "List of applications", body = inline(crate::models::PaginatedResponse<ApplicationWithRelations>)),
        (status = 500, description = "Internal server error")
    )
)]
async fn list(
    State(state): State<AppState>,
    Query(filters): Query<ApplicationFilters>,
) -> Result<impl axum::response::IntoResponse> {
    let params = PaginationParams {
        page: filters.page,
        per_page: filters.per_page,
        search: filters.search,
    };
    let result = application::list(
        &state.pool,
        &params,
        filters.status.as_deref(),
        filters.environment.as_deref(),
    )
    .await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/applications/{id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)")
    ),
    responses(
        (status = 200, description = "Application found", body = ApplicationWithRelations),
        (status = 404, description = "Application not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let id = complete_id_of_type(&state, &id, "application").await?;
    let result = application::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/applications/{id}/overview.md",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)")
    ),
    responses(
        (status = 200, description = "Markdown overview", content_type = "text/markdown", body = String),
        (status = 404, description = "Application not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_overview_md(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let id = complete_id_of_type(&state, &id, "application").await?;
    let app = application::get_with_relations(&state.pool, &id).await?;
    let md = app.body_md(&state);
    Ok(([(axum::http::header::CONTENT_TYPE, "text/markdown")], md))
}

#[utoipa::path(
    post,
    path = "/api/applications",
    tag = "applications",
    request_body = CreateApplication,
    responses(
        (status = 201, description = "Application created", body = Application),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateApplication>,
) -> Result<impl axum::response::IntoResponse> {
    let result = application::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(
    put,
    path = "/api/applications/{id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)")
    ),
    request_body = UpdateApplication,
    responses(
        (status = 200, description = "Application updated", body = Application),
        (status = 404, description = "Application not found"),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateApplication>,
) -> Result<impl axum::response::IntoResponse> {
    let id = complete_id_of_type(&state, &id, "application").await?;
    let result = application::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/api/applications/{id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)")
    ),
    responses(
        (status = 204, description = "Application deleted"),
        (status = 404, description = "Application not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let id = complete_id_of_type(&state, &id, "application").await?;
    application::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// Relationship handlers

#[utoipa::path(
    post,
    path = "/api/applications/{id}/infra/{infra_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("infra_id" = String, Path, description = "Infrastructure ID (prefix or full)")
    ),
    request_body = LinkInfra,
    responses(
        (status = 204, description = "Infrastructure linked successfully"),
        (status = 404, description = "Application or infrastructure not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn link_infra(
    State(state): State<AppState>,
    Path((app_id, infra_id)): Path<(String, String)>,
    Json(input): Json<LinkInfra>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let infra_id = complete_id_of_type(&state, &infra_id, "infra").await?;
    application::link_infra(&state.pool, &app_id, &infra_id, input.notes.as_deref()).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/applications/{id}/infra/{infra_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("infra_id" = String, Path, description = "Infrastructure ID (prefix or full)")
    ),
    responses(
        (status = 204, description = "Infrastructure unlinked successfully"),
        (status = 404, description = "Application or infrastructure not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn unlink_infra(
    State(state): State<AppState>,
    Path((app_id, infra_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let infra_id = complete_id_of_type(&state, &infra_id, "infra").await?;
    application::unlink_infra(&state.pool, &app_id, &infra_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/applications/{id}/services/{service_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("service_id" = String, Path, description = "Service ID (prefix or full)")
    ),
    request_body = LinkService,
    responses(
        (status = 204, description = "Service linked successfully"),
        (status = 404, description = "Application or service not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn link_service(
    State(state): State<AppState>,
    Path((app_id, service_id)): Path<(String, String)>,
    Json(input): Json<LinkService>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let service_id = complete_id_of_type(&state, &service_id, "service").await?;
    application::link_service(&state.pool, &app_id, &service_id, input.notes.as_deref()).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/applications/{id}/services/{service_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("service_id" = String, Path, description = "Service ID (prefix or full)")
    ),
    responses(
        (status = 204, description = "Service unlinked successfully"),
        (status = 404, description = "Application or service not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn unlink_service(
    State(state): State<AppState>,
    Path((app_id, service_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let service_id = complete_id_of_type(&state, &service_id, "service").await?;
    application::unlink_service(&state.pool, &app_id, &service_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/applications/{id}/domains/{domain_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("domain_id" = String, Path, description = "Domain ID (prefix or full)")
    ),
    request_body = LinkDomain,
    responses(
        (status = 204, description = "Domain linked successfully"),
        (status = 404, description = "Application or domain not found"),
        (status = 500, description = "Internal server error")
    )
)]
#[instrument(skip(state))]
async fn link_domain(
    State(state): State<AppState>,
    Path((app_id, domain_id)): Path<(String, String)>,
    Json(input): Json<LinkDomain>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let domain_id = complete_id_of_type(&state, &domain_id, "domain").await?;
    application::link_domain(&state.pool, &app_id, &domain_id, input.notes.as_deref()).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/applications/{id}/domains/{domain_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("domain_id" = String, Path, description = "Domain ID (prefix or full)")
    ),
    responses(
        (status = 204, description = "Domain unlinked successfully"),
        (status = 404, description = "Application or domain not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn unlink_domain(
    State(state): State<AppState>,
    Path((app_id, domain_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let domain_id = complete_id_of_type(&state, &domain_id, "domain").await?;
    application::unlink_domain(&state.pool, &app_id, &domain_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/applications/{id}/people/{person_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("person_id" = String, Path, description = "Person ID (prefix or full)")
    ),
    request_body = LinkPerson,
    responses(
        (status = 204, description = "Person linked successfully"),
        (status = 404, description = "Application or person not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn link_person(
    State(state): State<AppState>,
    Path((app_id, person_id)): Path<(String, String)>,
    Json(input): Json<LinkPerson>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let person_id = complete_id_of_type(&state, &person_id, "person").await?;
    application::link_person(
        &state.pool,
        &app_id,
        &person_id,
        &input.contribution_type,
        input.start_date.as_deref(),
        input.end_date.as_deref(),
        input.notes.as_deref(),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/applications/{id}/people/{person_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("person_id" = String, Path, description = "Person ID (prefix or full)")
    ),
    responses(
        (status = 204, description = "Person unlinked successfully"),
        (status = 404, description = "Application or person not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn unlink_person(
    State(state): State<AppState>,
    Path((app_id, person_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let person_id = complete_id_of_type(&state, &person_id, "person").await?;
    application::unlink_person(&state.pool, &app_id, &person_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/applications/{id}/shares/{share_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("share_id" = String, Path, description = "Network share ID (prefix or full)")
    ),
    request_body = LinkNetworkShare,
    responses(
        (status = 204, description = "Network share linked successfully"),
        (status = 404, description = "Application or network share not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn link_share(
    State(state): State<AppState>,
    Path((app_id, share_id)): Path<(String, String)>,
    Json(input): Json<LinkNetworkShare>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let share_id = complete_id_of_type(&state, &share_id, "network_share").await?;
    application::link_network_share(
        &state.pool,
        &app_id,
        &share_id,
        input.usage.as_deref(),
        input.mount_point.as_deref(),
        input.permissions.as_deref(),
        input.notes.as_deref(),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/applications/{id}/shares/{share_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("share_id" = String, Path, description = "Network share ID (prefix or full)")
    ),
    responses(
        (status = 204, description = "Network share unlinked successfully"),
        (status = 404, description = "Application or network share not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn unlink_share(
    State(state): State<AppState>,
    Path((app_id, share_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let share_id = complete_id_of_type(&state, &share_id, "network_share").await?;
    application::unlink_network_share(&state.pool, &app_id, &share_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/applications/{id}/stacks/{stack_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("stack_id" = String, Path, description = "Stack ID (prefix or full)")
    ),
    responses(
        (status = 204, description = "Stack linked successfully"),
        (status = 404, description = "Application or stack not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn link_stack(
    State(state): State<AppState>,
    Path((app_id, stack_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let stack_id = complete_id_of_type(&state, &stack_id, "stack").await?;
    application::link_stack(&state.pool, &app_id, &stack_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/applications/{id}/stacks/{stack_id}",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)"),
        ("stack_id" = String, Path, description = "Stack ID (prefix or full)")
    ),
    responses(
        (status = 204, description = "Stack unlinked successfully"),
        (status = 404, description = "Application or stack not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn unlink_stack(
    State(state): State<AppState>,
    Path((app_id, stack_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    let app_id = complete_id_of_type(&state, &app_id, "application").await?;
    let stack_id = complete_id_of_type(&state, &stack_id, "stack").await?;
    application::unlink_stack(&state.pool, &app_id, &stack_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/applications/{id}/sync-outline",
    tag = "applications",
    params(
        ("id" = String, Path, description = "Application ID (prefix or full)")
    ),
    responses(
        (status = 200, description = "Overview synced to Outline"),
        (status = 400, description = "Outline not configured or no outline_url set"),
        (status = 404, description = "Application not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn sync_outline(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let (Some(outline_url), Some(outline_api_key)) =
        (&state.config.outline_url, &state.config.outline_api_key)
    else {
        return Err(crate::Error::ValidationError(
            "Outline not configured (set OUTLINE_URL and OUTLINE_API_KEY)".to_string(),
        ));
    };

    let id = complete_id_of_type(&state, &id, "application").await?;
    let app = application::get_with_relations(&state.pool, &id).await?;

    let entity_outline_url = app.application.outline_url.as_deref().ok_or_else(|| {
        crate::Error::ValidationError("No Outline document linked to this application".to_string())
    })?;

    let doc_id = crate::outline::OutlineClient::extract_doc_id(entity_outline_url)?;
    let client = crate::outline::OutlineClient::new(outline_url, outline_api_key);

    let doc = client.get_document(&doc_id).await?;
    let new_text = app.splice_overview(&state, &doc.text);
    client.update_document(&doc_id, &new_text).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
