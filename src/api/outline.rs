use crate::{AppState, Result, outline};

use axum::{Router, extract::State, response::IntoResponse, routing::get};
use futures::{StreamExt as _, stream};
use sqlx::prelude::FromRow;
use tracing::info;

pub fn routes() -> Router<AppState> {
    Router::new().route("/sync", get(sync_all))
}

#[derive(Debug, FromRow)]
struct Id {
    id: String,
}

async fn sync_all(State(state): State<AppState>) -> Result<impl IntoResponse> {
    // sync all applications
    let app_ids = sqlx::query_as::<_, Id>("SELECT id FROM application;")
        .fetch_all(&state.pool)
        .await?;
    let service_ids = sqlx::query_as::<_, Id>("SELECT id FROM service;")
        .fetch_all(&state.pool)
        .await?;

    let _res: Vec<_> = stream::iter(app_ids)
        .map(|app| {
            let state = state.clone();
            let app_id = app.id;
            async move {
                info!("Syncing Appliation {app_id}");
                outline::sync_application(&state, &app_id).await
            }
        })
        .buffer_unordered(1)
        .collect()
        .await;

    let _res: Vec<_> = stream::iter(service_ids)
        .map(|app| {
            let state = state.clone();
            let serv_id = app.id;
            async move {
                info!("Syncing Service {serv_id}");
                outline::sync_service(&state, &serv_id).await
            }
        })
        .buffer_unordered(1)
        .collect()
        .await;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
