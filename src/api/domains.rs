use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::models::{CreateDomain, PaginationParams, UpdateDomain, DomainWithRelations, Domain};
use crate::service::domain;
use crate::{AppState, Result};

#[derive(Debug, Deserialize, Default)]
pub struct DomainFilters {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
}

#[utoipa::path(
    get,
    path = "/api/domains",
    tag = "domains",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("per_page" = Option<u32>, Query, description = "Items per page (max 100)"),
        ("search" = Option<String>, Query, description = "Search query"),
    ),
    responses(
        (status = 200, description = "List of domains", body = inline(crate::models::PaginatedResponse<DomainWithRelations>)),
        (status = 500, description = "Internal server error")
    )
)]
async fn list(
    State(state): State<AppState>,
    Query(filters): Query<DomainFilters>,
) -> Result<impl axum::response::IntoResponse> {
    let params = PaginationParams {
        page: filters.page,
        per_page: filters.per_page,
        search: filters.search,
    };
    let result = domain::list(&state.pool, &params).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/domains/{id}",
    tag = "domains",
    params(
        ("id" = String, Path, description = "Domain ID")
    ),
    responses(
        (status = 200, description = "Domain found", body = DomainWithRelations),
        (status = 404, description = "Domain not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = domain::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/domains",
    tag = "domains",
    request_body = CreateDomain,
    responses(
        (status = 201, description = "Domain created", body = Domain),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateDomain>,
) -> Result<impl axum::response::IntoResponse> {
    let result = domain::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(
    put,
    path = "/api/domains/{id}",
    tag = "domains",
    params(
        ("id" = String, Path, description = "Domain ID")
    ),
    request_body = UpdateDomain,
    responses(
        (status = 200, description = "Domain updated", body = Domain),
        (status = 404, description = "Domain not found"),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateDomain>,
) -> Result<impl axum::response::IntoResponse> {
    let result = domain::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/api/domains/{id}",
    tag = "domains",
    params(
        ("id" = String, Path, description = "Domain ID")
    ),
    responses(
        (status = 204, description = "Domain deleted"),
        (status = 404, description = "Domain not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    domain::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
