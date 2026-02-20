use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures::stream::{self, Stream, StreamExt};
use serde::Deserialize;
use tracing::debug;

use crate::models::{
    CreateHealthcheck, Healthcheck, HealthcheckExecuteResult, HealthcheckWithRelations,
    KumaEndpoint, KumaMonitor, PaginationParams, UpdateHealthcheck, UptimeEvent,
};
use crate::service::healthcheck;
use crate::{AppState, Result, kuma};

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
        .route("/sync/kuma", post(sync_kuma_all))
        .route("/kuma-endpoint", get(kuma_endpoint))
        .route("/sync/kuma/{id}", post(sync_kuma_one))
        .route("/uptime/stream", get(uptime_stream))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
        .route("/{id}/execute", get(execute))
}

#[utoipa::path(
    get,
    path = "/api/healthchecks",
    tag = "healthchecks",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("per_page" = Option<u32>, Query, description = "Items per page (max 100)"),
        ("search" = Option<String>, Query, description = "Search query"),
        ("application_id" = Option<String>, Query, description = "Filter by application ID"),
        ("service_id" = Option<String>, Query, description = "Filter by service ID"),
        ("is_enabled" = Option<bool>, Query, description = "Filter by enabled status"),
    ),
    responses(
        (status = 200, description = "List of healthchecks", body = inline(crate::models::PaginatedResponse<HealthcheckWithRelations>)),
        (status = 500, description = "Internal server error")
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/healthchecks/{id}",
    tag = "healthchecks",
    params(
        ("id" = String, Path, description = "Healthcheck ID")
    ),
    responses(
        (status = 200, description = "Healthcheck found", body = HealthcheckWithRelations),
        (status = 404, description = "Healthcheck not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = healthcheck::get_with_relations(&state.pool, &id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/healthchecks",
    tag = "healthchecks",
    request_body = CreateHealthcheck,
    responses(
        (status = 201, description = "Healthcheck created", body = Healthcheck),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateHealthcheck>,
) -> Result<impl axum::response::IntoResponse> {
    let result = healthcheck::create(&state.pool, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

#[utoipa::path(
    put,
    path = "/api/healthchecks/{id}",
    tag = "healthchecks",
    params(
        ("id" = String, Path, description = "Healthcheck ID")
    ),
    request_body = UpdateHealthcheck,
    responses(
        (status = 200, description = "Healthcheck updated", body = Healthcheck),
        (status = 404, description = "Healthcheck not found"),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateHealthcheck>,
) -> Result<impl axum::response::IntoResponse> {
    let result = healthcheck::update(&state.pool, &id, input).await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/api/healthchecks/{id}",
    tag = "healthchecks",
    params(
        ("id" = String, Path, description = "Healthcheck ID")
    ),
    responses(
        (status = 204, description = "Healthcheck deleted"),
        (status = 404, description = "Healthcheck not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    healthcheck::delete(&state.pool, &id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/healthchecks/{id}/execute",
    tag = "healthchecks",
    params(
        ("id" = String, Path, description = "Healthcheck ID")
    ),
    responses(
        (status = 200, description = "Healthcheck execution result", body = HealthcheckExecuteResult),
        (status = 404, description = "Healthcheck not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn execute(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    let result = healthcheck::execute(&state.pool, &id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/kuma-endpoint",
    tag = "healthchecks",
    responses(
        (status = 200, description = "The base URL of the kuma endpoint this server is polling", body = inline(KumaEndpoint)),
        (status = 500, description = "Internal server error")
    )
)]
async fn kuma_endpoint(State(state): State<AppState>) -> Result<Json<KumaEndpoint>> {
    let url = state.config.kuma_url.to_string();
    Ok(Json(KumaEndpoint { url }))
}

#[utoipa::path(
    get,
    path = "/api/healthchecks/export/kuma",
    tag = "healthchecks",
    responses(
        (status = 200, description = "Export healthchecks for Kuma Uptime", body = Vec<KumaMonitor>),
        (status = 500, description = "Internal server error")
    )
)]
async fn export_kuma(State(state): State<AppState>) -> Result<impl axum::response::IntoResponse> {
    let result = healthcheck::export_kuma(&state.pool).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/healthchecks/sync/kuma/{id}",
    tag = "healthchecks",
    params(
        ("id" = String, Path, description = "Healthcheck ID to sync")
    ),
    responses(
        (status = 204, description = "Healthcheck synced to Kuma"),
        (status = 404, description = "Healthcheck not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn sync_kuma_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl axum::response::IntoResponse> {
    // rust_socketio::Client is !Send, so we run the sync on a dedicated
    // blocking thread that owns its own async context.
    let handle = tokio::runtime::Handle::current();
    let state_clone = state.clone();
    tokio::task::spawn_blocking(move || {
        handle.block_on(kuma::sync_healthcheck_to_kuma(state_clone, &id))
    })
    .await
    .map_err(|e| crate::Error::InternalError(e.to_string()))??;

    // Notify the persistent poller to reconnect and pick up new kuma_ids
    let _ = state.kuma_refresh_tx.send(());

    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/healthchecks/sync/kuma",
    tag = "healthchecks",
    responses(
        (status = 204, description = "All healthchecks synced to Kuma"),
        (status = 500, description = "Internal server error")
    )
)]
async fn sync_kuma_all(State(state): State<AppState>) -> Result<impl axum::response::IntoResponse> {
    // rust_socketio::Client is !Send, so we run the sync on a dedicated
    // blocking thread that owns its own async context.
    let handle = tokio::runtime::Handle::current();
    let state_clone = state.clone();
    tokio::task::spawn_blocking(move || {
        handle.block_on(kuma::sync_healthchecks_to_kuma(state_clone))
    })
    .await
    .map_err(|e| crate::Error::InternalError(e.to_string()))??;

    // Notify the persistent poller to reconnect and pick up new kuma_ids
    let _ = state.kuma_refresh_tx.send(());

    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn uptime_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = std::result::Result<Event, axum::Error>>> {
    // Subscribe FIRST, then read snapshot — prevents missed events in the gap.
    let rx = state.uptime_tx.subscribe();

    // Build the initial snapshot
    // NOTE: this snapshot is too large. It is about 1.8MB of data and seems to take quite a while
    // to get to the client
    debug!("Getting uptime state snapshot");
    let snapshot = {
        let read = state.uptime_state.read().await;
        UptimeEvent::Snapshot {
            monitors: read.clone(),
        }
    };

    let snapshot_json = serde_json::to_string(&snapshot)
        .unwrap_or_else(|_| r#"{"type":"snapshot","monitors":{}}"#.to_string());
    let snapshot_event = Ok::<Event, axum::Error>(Event::default().data(snapshot_json));

    // Chain: one snapshot event, then the live broadcast stream
    let initial = stream::once(async move { snapshot_event });
    let live = broadcast_to_stream(rx);
    let combined = initial.chain(live);

    debug!("Start sending stream");

    Sse::new(combined).keep_alive(KeepAlive::default())
}

/// Converts a `broadcast::Receiver` into a `Stream` of SSE events.
fn broadcast_to_stream(
    rx: tokio::sync::broadcast::Receiver<UptimeEvent>,
) -> impl Stream<Item = std::result::Result<Event, axum::Error>> {
    futures::stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if let Ok(json) = serde_json::to_string(&event) {
                        return Some((Ok(Event::default().data(json)), rx));
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("SSE client lagged by {n} events, continuing");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    })
}
