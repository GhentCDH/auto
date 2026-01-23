use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::models::{CreateNote, PaginationParams, UpdateNote};
use crate::service::note;
use crate::{AppState, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
}

#[derive(Debug, Deserialize)]
struct ListParams {
    entity_type: String,
    entity_id: String,
    #[serde(flatten)]
    pagination: PaginationParams,
}

async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<impl axum::response::IntoResponse> {
    let result = note::list_for_entity(
        &state.pool,
        &params.entity_type,
        &params.entity_id,
        &params.pagination,
    )
    .await?;
    Ok(Json(result))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = note::get(&state.pool, &id).await?;
    Ok(Json(result))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateNote>,
) -> Result<impl axum::response::IntoResponse> {
    let result = note::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateNote>,
) -> Result<impl axum::response::IntoResponse> {
    let result = note::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    note::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
