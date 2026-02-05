use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::models::{CreateHealthcheck, PaginationParams, UpdateHealthcheck};
use crate::service::healthcheck;
use crate::{AppState, Result};

#[derive(Debug, Deserialize, Default)]
pub struct HealthcheckFilters {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub application_id: Option<String>,
    pub service_id: Option<String>,
    pub is_enabled: Option<bool>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/export/kuma", get(export_kuma))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
        .route("/{id}/execute", get(execute))
}

async fn list(
    State(state): State<AppState>,
    Query(filters): Query<HealthcheckFilters>,
) -> Result<impl axum::response::IntoResponse> {
    let params = PaginationParams {
        page: filters.page,
        per_page: filters.per_page,
        search: filters.search,
    };
    let result = healthcheck::list(
        &state.pool,
        &params,
        filters.application_id.as_deref(),
        filters.service_id.as_deref(),
        filters.is_enabled,
    )
    .await?;
    Ok(Json(result))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = healthcheck::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateHealthcheck>,
) -> Result<impl axum::response::IntoResponse> {
    let result = healthcheck::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateHealthcheck>,
) -> Result<impl axum::response::IntoResponse> {
    let result = healthcheck::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    healthcheck::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn execute(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = healthcheck::execute(&state.pool, &id).await?;
    Ok(Json(result))
}

async fn export_kuma(State(state): State<AppState>) -> Result<impl axum::response::IntoResponse> {
    let result = healthcheck::export_kuma(&state.pool).await?;
    Ok(Json(result))
}
