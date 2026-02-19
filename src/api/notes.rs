use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::models::{CreateNote, PaginationParams, UpdateNote, Note};
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

#[utoipa::path(
    get,
    path = "/api/notes",
    tag = "notes",
    params(
        ("entity_type" = String, Query, description = "Entity type (application, service, etc)"),
        ("entity_id" = String, Query, description = "Entity ID"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("per_page" = Option<u32>, Query, description = "Items per page (max 100)"),
        ("search" = Option<String>, Query, description = "Search query"),
    ),
    responses(
        (status = 200, description = "List of notes", body = inline(crate::models::PaginatedResponse<Note>)),
        (status = 500, description = "Internal server error")
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/notes/{id}",
    tag = "notes",
    params(
        ("id" = String, Path, description = "Note ID")
    ),
    responses(
        (status = 200, description = "Note found", body = Note),
        (status = 404, description = "Note not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = note::get(&state.pool, &id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/notes",
    tag = "notes",
    request_body = CreateNote,
    responses(
        (status = 201, description = "Note created", body = Note),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateNote>,
) -> Result<impl axum::response::IntoResponse> {
    let result = note::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(
    put,
    path = "/api/notes/{id}",
    tag = "notes",
    params(
        ("id" = String, Path, description = "Note ID")
    ),
    request_body = UpdateNote,
    responses(
        (status = 200, description = "Note updated", body = Note),
        (status = 404, description = "Note not found"),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateNote>,
) -> Result<impl axum::response::IntoResponse> {
    let result = note::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/api/notes/{id}",
    tag = "notes",
    params(
        ("id" = String, Path, description = "Note ID")
    ),
    responses(
        (status = 204, description = "Note deleted"),
        (status = 404, description = "Note not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    note::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
