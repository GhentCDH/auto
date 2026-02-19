use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::models::{CreatePerson, PaginationParams, UpdatePerson, PersonWithRelations, Person};
use crate::service::person;
use crate::{AppState, Result};

#[derive(Debug, Deserialize, Default)]
pub struct PersonFilters {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
}

#[utoipa::path(
    get,
    path = "/api/people",
    tag = "people",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("per_page" = Option<u32>, Query, description = "Items per page (max 100)"),
        ("search" = Option<String>, Query, description = "Search query"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
    ),
    responses(
        (status = 200, description = "List of people", body = inline(crate::models::PaginatedResponse<PersonWithRelations>)),
        (status = 500, description = "Internal server error")
    )
)]
async fn list(
    State(state): State<AppState>,
    Query(filters): Query<PersonFilters>,
) -> Result<impl axum::response::IntoResponse> {
    let params = PaginationParams {
        page: filters.page,
        per_page: filters.per_page,
        search: filters.search,
    };
    let result = person::list(&state.pool, &params, filters.is_active).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/people/{id}",
    tag = "people",
    params(
        ("id" = String, Path, description = "Person ID")
    ),
    responses(
        (status = 200, description = "Person found", body = PersonWithRelations),
        (status = 404, description = "Person not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = person::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/people",
    tag = "people",
    request_body = CreatePerson,
    responses(
        (status = 201, description = "Person created", body = Person),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreatePerson>,
) -> Result<impl axum::response::IntoResponse> {
    let result = person::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(
    put,
    path = "/api/people/{id}",
    tag = "people",
    params(
        ("id" = String, Path, description = "Person ID")
    ),
    request_body = UpdatePerson,
    responses(
        (status = 200, description = "Person updated", body = Person),
        (status = 404, description = "Person not found"),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdatePerson>,
) -> Result<impl axum::response::IntoResponse> {
    let result = person::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/api/people/{id}",
    tag = "people",
    params(
        ("id" = String, Path, description = "Person ID")
    ),
    responses(
        (status = 204, description = "Person deleted"),
        (status = 404, description = "Person not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    person::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
