use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::models::{CreateNetworkShare, PaginationParams, UpdateNetworkShare};
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

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = network_share::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateNetworkShare>,
) -> Result<impl axum::response::IntoResponse> {
    let result = network_share::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateNetworkShare>,
) -> Result<impl axum::response::IntoResponse> {
    let result = network_share::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    network_share::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
