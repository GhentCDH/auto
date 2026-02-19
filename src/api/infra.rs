use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::models::{CreateInfra, PaginationParams, UpdateInfra, InfraWithRelations, Infra};
use crate::service::infra;
use crate::{AppState, Result};

#[derive(Debug, Deserialize, Default)]
pub struct InfraFilters {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    #[serde(rename = "type")]
    pub infra_type: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
}

#[utoipa::path(
    get,
    path = "/api/infra",
    tag = "infra",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("per_page" = Option<u32>, Query, description = "Items per page (max 100)"),
        ("search" = Option<String>, Query, description = "Search query"),
        ("type" = Option<String>, Query, description = "Filter by infrastructure type"),
    ),
    responses(
        (status = 200, description = "List of infrastructure", body = inline(crate::models::PaginatedResponse<InfraWithRelations>)),
        (status = 500, description = "Internal server error")
    )
)]
async fn list(
    State(state): State<AppState>,
    Query(filters): Query<InfraFilters>,
) -> Result<impl axum::response::IntoResponse> {
    let params = PaginationParams {
        page: filters.page,
        per_page: filters.per_page,
        search: filters.search,
    };
    let result = infra::list(&state.pool, &params, filters.infra_type.as_deref()).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/infra/{id}",
    tag = "infra",
    params(
        ("id" = String, Path, description = "Infrastructure ID")
    ),
    responses(
        (status = 200, description = "Infrastructure found", body = InfraWithRelations),
        (status = 404, description = "Infrastructure not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = infra::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/infra",
    tag = "infra",
    request_body = CreateInfra,
    responses(
        (status = 201, description = "Infrastructure created", body = Infra),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateInfra>,
) -> Result<impl axum::response::IntoResponse> {
    let result = infra::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(
    put,
    path = "/api/infra/{id}",
    tag = "infra",
    params(
        ("id" = String, Path, description = "Infrastructure ID")
    ),
    request_body = UpdateInfra,
    responses(
        (status = 200, description = "Infrastructure updated", body = Infra),
        (status = 404, description = "Infrastructure not found"),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateInfra>,
) -> Result<impl axum::response::IntoResponse> {
    let result = infra::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/api/infra/{id}",
    tag = "infra",
    params(
        ("id" = String, Path, description = "Infrastructure ID")
    ),
    responses(
        (status = 204, description = "Infrastructure deleted"),
        (status = 404, description = "Infrastructure not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    infra::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
