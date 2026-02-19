use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures::stream::{self, Stream, StreamExt};
use serde::Deserialize;

use crate::models::{CreateHealthcheck, PaginationParams, UpdateHealthcheck, UptimeEvent};
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
        .route("/sync/kuma/{id}", post(sync_kuma_one))
        .route("/uptime/stream", get(uptime_stream))
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

// ── SSE uptime stream ───────────────────────────────────────────────

async fn uptime_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = std::result::Result<Event, axum::Error>>> {
    // Subscribe FIRST, then read snapshot — prevents missed events in the gap.
    let rx = state.uptime_tx.subscribe();

    // Build the initial snapshot
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
