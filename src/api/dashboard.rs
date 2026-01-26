use axum::{Json, Router, extract::State, routing::get};

use crate::service::dashboard;
use crate::{AppState, Result};

pub fn routes() -> Router<AppState> {
    Router::new().route("/stats", get(stats))
}

async fn stats(State(state): State<AppState>) -> Result<impl axum::response::IntoResponse> {
    let result = dashboard::get_stats(&state.pool).await?;
    Ok(Json(result))
}
