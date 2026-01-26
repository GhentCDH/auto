use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};

use crate::models::{CreateNetworkShare, PaginationParams, UpdateNetworkShare};
use crate::service::network_share;
use crate::{AppState, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
}

async fn list(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl axum::response::IntoResponse> {
    let result = network_share::list(&state.pool, &params).await?;
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
