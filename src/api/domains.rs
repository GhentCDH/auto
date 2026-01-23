use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};

use crate::models::{CreateDomain, PaginationParams, UpdateDomain};
use crate::service::domain;
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
    let result = domain::list(&state.pool, &params).await?;
    Ok(Json(result))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = domain::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateDomain>,
) -> Result<impl axum::response::IntoResponse> {
    let result = domain::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateDomain>,
) -> Result<impl axum::response::IntoResponse> {
    let result = domain::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    domain::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
