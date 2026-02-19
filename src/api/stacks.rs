use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};

use crate::models::{CreateStack, PaginationParams, UpdateStack, StackWithRelations, Stack};
use crate::service::stack;
use crate::{AppState, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
}

#[utoipa::path(
    get,
    path = "/api/stacks",
    tag = "stacks",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("per_page" = Option<u32>, Query, description = "Items per page (max 100)"),
        ("search" = Option<String>, Query, description = "Search query"),
    ),
    responses(
        (status = 200, description = "List of stacks", body = inline(crate::models::PaginatedResponse<Stack>)),
        (status = 500, description = "Internal server error")
    )
)]
async fn list(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl axum::response::IntoResponse> {
    let result = stack::list(&state.pool, &params).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/stacks/{id}",
    tag = "stacks",
    params(
        ("id" = String, Path, description = "Stack ID")
    ),
    responses(
        (status = 200, description = "Stack found", body = StackWithRelations),
        (status = 404, description = "Stack not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = stack::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/stacks",
    tag = "stacks",
    request_body = CreateStack,
    responses(
        (status = 201, description = "Stack created", body = Stack),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateStack>,
) -> Result<impl axum::response::IntoResponse> {
    let result = stack::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(
    put,
    path = "/api/stacks/{id}",
    tag = "stacks",
    params(
        ("id" = String, Path, description = "Stack ID")
    ),
    request_body = UpdateStack,
    responses(
        (status = 200, description = "Stack updated", body = Stack),
        (status = 404, description = "Stack not found"),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateStack>,
) -> Result<impl axum::response::IntoResponse> {
    let result = stack::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/api/stacks/{id}",
    tag = "stacks",
    params(
        ("id" = String, Path, description = "Stack ID")
    ),
    responses(
        (status = 204, description = "Stack deleted"),
        (status = 404, description = "Stack not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    stack::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
