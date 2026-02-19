use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::models::{CreateNetworkShare, PaginationParams, UpdateNetworkShare, NetworkShareWithRelations, NetworkShare};
use crate::service::network_share;
use crate::{AppState, Result};

#[derive(Debug, Deserialize, Default)]
pub struct ShareFilters {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub status: Option<String>,
    pub share_type: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
}

#[utoipa::path(
    get,
    path = "/api/shares",
    tag = "shares",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("per_page" = Option<u32>, Query, description = "Items per page (max 100)"),
        ("search" = Option<String>, Query, description = "Search query"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("share_type" = Option<String>, Query, description = "Filter by share type (smb, nfs)"),
    ),
    responses(
        (status = 200, description = "List of network shares", body = inline(crate::models::PaginatedResponse<NetworkShareWithRelations>)),
        (status = 500, description = "Internal server error")
    )
)]
async fn list(
    State(state): State<AppState>,
    Query(filters): Query<ShareFilters>,
) -> Result<impl axum::response::IntoResponse> {
    let params = PaginationParams {
        page: filters.page,
        per_page: filters.per_page,
        search: filters.search,
    };
    let result = network_share::list(
        &state.pool,
        &params,
        filters.status.as_deref(),
        filters.share_type.as_deref(),
    )
    .await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/shares/{id}",
    tag = "shares",
    params(
        ("id" = String, Path, description = "Network share ID")
    ),
    responses(
        (status = 200, description = "Network share found", body = NetworkShareWithRelations),
        (status = 404, description = "Network share not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = network_share::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/shares",
    tag = "shares",
    request_body = CreateNetworkShare,
    responses(
        (status = 201, description = "Network share created", body = NetworkShare),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateNetworkShare>,
) -> Result<impl axum::response::IntoResponse> {
    let result = network_share::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(
    put,
    path = "/api/shares/{id}",
    tag = "shares",
    params(
        ("id" = String, Path, description = "Network share ID")
    ),
    request_body = UpdateNetworkShare,
    responses(
        (status = 200, description = "Network share updated", body = NetworkShare),
        (status = 404, description = "Network share not found"),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateNetworkShare>,
) -> Result<impl axum::response::IntoResponse> {
    let result = network_share::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/api/shares/{id}",
    tag = "shares",
    params(
        ("id" = String, Path, description = "Network share ID")
    ),
    responses(
        (status = 204, description = "Network share deleted"),
        (status = 404, description = "Network share not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    network_share::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
