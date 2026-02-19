use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use tracing::info;

use crate::models::{CreateService, LinkInfra, PaginationParams, UpdateService, ServiceWithRelations, Service};
use crate::service::service;
use crate::{AppState, Result};

#[derive(Debug, Deserialize, Default)]
pub struct ServiceFilters {
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
        .route(
            "/{id}/infra/{infra_id}",
            post(link_infra).delete(unlink_infra),
        )
}

#[utoipa::path(
    get,
    path = "/api/services",
    tag = "services",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("per_page" = Option<u32>, Query, description = "Items per page (max 100)"),
        ("search" = Option<String>, Query, description = "Search query"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("environment" = Option<String>, Query, description = "Filter by environment"),
    ),
    responses(
        (status = 200, description = "List of services", body = inline(crate::models::PaginatedResponse<ServiceWithRelations>)),
        (status = 500, description = "Internal server error")
    )
)]
async fn list(
    State(state): State<AppState>,
    Query(filters): Query<ServiceFilters>,
) -> Result<impl axum::response::IntoResponse> {
    info!("filters.environment: {:?}", filters.status);
    let params = PaginationParams {
        page: filters.page,
        per_page: filters.per_page,
        search: filters.search,
    };
    let result = service::list(
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
    path = "/api/services/{id}",
    tag = "services",
    params(
        ("id" = String, Path, description = "Service ID")
    ),
    responses(
        (status = 200, description = "Service found", body = ServiceWithRelations),
        (status = 404, description = "Service not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = service::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/services",
    tag = "services",
    request_body = CreateService,
    responses(
        (status = 201, description = "Service created", body = Service),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateService>,
) -> Result<impl axum::response::IntoResponse> {
    let result = service::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(
    put,
    path = "/api/services/{id}",
    tag = "services",
    params(
        ("id" = String, Path, description = "Service ID")
    ),
    request_body = UpdateService,
    responses(
        (status = 200, description = "Service updated", body = Service),
        (status = 404, description = "Service not found"),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateService>,
) -> Result<impl axum::response::IntoResponse> {
    let result = service::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/api/services/{id}",
    tag = "services",
    params(
        ("id" = String, Path, description = "Service ID")
    ),
    responses(
        (status = 204, description = "Service deleted"),
        (status = 404, description = "Service not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    service::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// Relationship handlers

#[utoipa::path(
    post,
    path = "/api/services/{id}/infra/{infra_id}",
    tag = "services",
    params(
        ("id" = String, Path, description = "Service ID"),
        ("infra_id" = String, Path, description = "Infrastructure ID")
    ),
    request_body = LinkInfra,
    responses(
        (status = 204, description = "Infrastructure linked successfully"),
        (status = 404, description = "Service or infrastructure not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn link_infra(
    State(state): State<AppState>,
    Path((service_id, infra_id)): Path<(String, String)>,
    Json(input): Json<LinkInfra>,
) -> Result<impl axum::response::IntoResponse> {
    service::link_infra(&state.pool, &service_id, &infra_id, input.notes.as_deref()).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/services/{id}/infra/{infra_id}",
    tag = "services",
    params(
        ("id" = String, Path, description = "Service ID"),
        ("infra_id" = String, Path, description = "Infrastructure ID")
    ),
    responses(
        (status = 204, description = "Infrastructure unlinked successfully"),
        (status = 404, description = "Service or infrastructure not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn unlink_infra(
    State(state): State<AppState>,
    Path((service_id, infra_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    service::unlink_infra(&state.pool, &service_id, &infra_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
