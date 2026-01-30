use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use tracing::instrument;

use crate::models::{
    CreateApplication, LinkDomain, LinkInfra, LinkNetworkShare, LinkPerson, LinkService,
    PaginationParams, UpdateApplication,
};
use crate::service::application;
use crate::{AppState, Result};

#[derive(Debug, Deserialize, Default)]
pub struct ApplicationFilters {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub status: Option<String>,
    pub environment: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
        // Relationship management
        .route(
            "/{id}/infra/{infra_id}",
            post(link_infra).delete(unlink_infra),
        )
        .route(
            "/{id}/services/{service_id}",
            post(link_service).delete(unlink_service),
        )
        .route(
            "/{id}/domains/{domain_id}",
            post(link_domain).delete(unlink_domain),
        )
        .route(
            "/{id}/people/{person_id}",
            post(link_person).delete(unlink_person),
        )
        .route(
            "/{id}/shares/{share_id}",
            post(link_share).delete(unlink_share),
        )
        .route(
            "/{id}/stacks/{stack_id}",
            post(link_stack).delete(unlink_stack),
        )
}

async fn list(
    State(state): State<AppState>,
    Query(filters): Query<ApplicationFilters>,
) -> Result<impl axum::response::IntoResponse> {
    let params = PaginationParams {
        page: filters.page,
        per_page: filters.per_page,
        search: filters.search,
    };
    let result = application::list(
        &state.pool,
        &params,
        filters.status.as_deref(),
        filters.environment.as_deref(),
    )
    .await?;
    Ok(Json(result))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = application::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateApplication>,
) -> Result<impl axum::response::IntoResponse> {
    let result = application::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateApplication>,
) -> Result<impl axum::response::IntoResponse> {
    let result = application::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    application::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// Relationship handlers

async fn link_infra(
    State(state): State<AppState>,
    Path((app_id, infra_id)): Path<(String, String)>,
    Json(input): Json<LinkInfra>,
) -> Result<impl axum::response::IntoResponse> {
    application::link_infra(&state.pool, &app_id, &infra_id, input.notes.as_deref()).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn unlink_infra(
    State(state): State<AppState>,
    Path((app_id, infra_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    application::unlink_infra(&state.pool, &app_id, &infra_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn link_service(
    State(state): State<AppState>,
    Path((app_id, service_id)): Path<(String, String)>,
    Json(input): Json<LinkService>,
) -> Result<impl axum::response::IntoResponse> {
    application::link_service(&state.pool, &app_id, &service_id, input.notes.as_deref()).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn unlink_service(
    State(state): State<AppState>,
    Path((app_id, service_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    application::unlink_service(&state.pool, &app_id, &service_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[instrument(skip(state))]
async fn link_domain(
    State(state): State<AppState>,
    Path((app_id, domain_id)): Path<(String, String)>,
    Json(input): Json<LinkDomain>,
) -> Result<impl axum::response::IntoResponse> {
    application::link_domain(&state.pool, &app_id, &domain_id, input.notes.as_deref()).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn unlink_domain(
    State(state): State<AppState>,
    Path((app_id, domain_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    application::unlink_domain(&state.pool, &app_id, &domain_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn link_person(
    State(state): State<AppState>,
    Path((app_id, person_id)): Path<(String, String)>,
    Json(input): Json<LinkPerson>,
) -> Result<impl axum::response::IntoResponse> {
    application::link_person(
        &state.pool,
        &app_id,
        &person_id,
        &input.contribution_type,
        input.start_date.as_deref(),
        input.end_date.as_deref(),
        input.notes.as_deref(),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn unlink_person(
    State(state): State<AppState>,
    Path((app_id, person_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    application::unlink_person(&state.pool, &app_id, &person_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn link_share(
    State(state): State<AppState>,
    Path((app_id, share_id)): Path<(String, String)>,
    Json(input): Json<LinkNetworkShare>,
) -> Result<impl axum::response::IntoResponse> {
    application::link_network_share(
        &state.pool,
        &app_id,
        &share_id,
        input.usage.as_deref(),
        input.mount_point.as_deref(),
        input.permissions.as_deref(),
        input.notes.as_deref(),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn unlink_share(
    State(state): State<AppState>,
    Path((app_id, share_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    application::unlink_network_share(&state.pool, &app_id, &share_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn link_stack(
    State(state): State<AppState>,
    Path((app_id, stack_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    application::link_stack(&state.pool, &app_id, &stack_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn unlink_stack(
    State(state): State<AppState>,
    Path((app_id, stack_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    application::unlink_stack(&state.pool, &app_id, &stack_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
