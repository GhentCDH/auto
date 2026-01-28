use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};

use crate::models::{CreateService, LinkInfra, PaginationParams, UpdateService};
use crate::service::service;
use crate::{AppState, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
        .route(
            "/{id}/infra/{infra_id}",
            post(link_infra).delete(unlink_infra),
        )
}

async fn list(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl axum::response::IntoResponse> {
    let result = service::list(&state.pool, &params).await?;
    Ok(Json(result))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = service::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateService>,
) -> Result<impl axum::response::IntoResponse> {
    let result = service::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateService>,
) -> Result<impl axum::response::IntoResponse> {
    let result = service::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    service::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// Relationship handlers

async fn link_infra(
    State(state): State<AppState>,
    Path((service_id, infra_id)): Path<(String, String)>,
    Json(input): Json<LinkInfra>,
) -> Result<impl axum::response::IntoResponse> {
    service::link_infra(&state.pool, &service_id, &infra_id, input.notes.as_deref()).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn unlink_infra(
    State(state): State<AppState>,
    Path((service_id, infra_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    service::unlink_infra(&state.pool, &service_id, &infra_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
