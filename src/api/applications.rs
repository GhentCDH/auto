use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use tracing::instrument;

use crate::models::{
    CreateApplication, LinkClient, LinkDomain, LinkHost, LinkNetworkShare, LinkPerson,
    PaginationParams, UpdateApplication,
};
use crate::service::application;
use crate::{AppState, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
        // Relationship management
        .route("/{id}/hosts/{host_id}", post(link_host).delete(unlink_host))
        .route(
            "/{id}/domains/{domain_id}",
            post(link_domain).delete(unlink_domain),
        )
        .route(
            "/{id}/people/{person_id}",
            post(link_person).delete(unlink_person),
        )
        .route(
            "/{id}/clients/{client_id}",
            post(link_client).delete(unlink_client),
        )
        .route(
            "/{id}/shares/{share_id}",
            post(link_share).delete(unlink_share),
        )
}

async fn list(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl axum::response::IntoResponse> {
    let result = application::list(&state.pool, &params).await?;
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

async fn link_host(
    State(state): State<AppState>,
    Path((app_id, host_id)): Path<(String, String)>,
    Json(input): Json<LinkHost>,
) -> Result<impl axum::response::IntoResponse> {
    application::link_host(
        &state.pool,
        &app_id,
        &host_id,
        &input.role,
        input.notes.as_deref(),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn unlink_host(
    State(state): State<AppState>,
    Path((app_id, host_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    application::unlink_host(&state.pool, &app_id, &host_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[instrument(skip(state))]
async fn link_domain(
    State(state): State<AppState>,
    Path((app_id, domain_id)): Path<(String, String)>,
    Json(input): Json<LinkDomain>,
) -> Result<impl axum::response::IntoResponse> {
    application::link_domain(
        &state.pool,
        &app_id,
        &domain_id,
        &input.record_type,
        input.target.as_deref(),
        input.target_host_id.as_deref(),
        input.is_primary,
        input.notes.as_deref(),
    )
    .await?;
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

async fn link_client(
    State(state): State<AppState>,
    Path((app_id, client_id)): Path<(String, String)>,
    Json(input): Json<LinkClient>,
) -> Result<impl axum::response::IntoResponse> {
    application::link_client(
        &state.pool,
        &app_id,
        &client_id,
        &input.relationship_type,
        input.contract_ref.as_deref(),
        input.notes.as_deref(),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn unlink_client(
    State(state): State<AppState>,
    Path((app_id, client_id)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse> {
    application::unlink_client(&state.pool, &app_id, &client_id).await?;
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
