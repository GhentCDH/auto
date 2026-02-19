use axum::{Json, Router, extract::State, routing::get};

use crate::service::dashboard::{self, DashboardStats};
use crate::{AppState, Result};

pub fn routes() -> Router<AppState> {
    Router::new().route("/stats", get(stats))
}

#[utoipa::path(
    get,
    path = "/api/dashboard/stats",
    tag = "dashboard",
    responses(
        (status = 200, description = "Dashboard statistics", body = DashboardStats),
        (status = 500, description = "Internal server error")
    )
)]
async fn stats(State(state): State<AppState>) -> Result<impl axum::response::IntoResponse> {
    let result = dashboard::get_stats(&state.pool).await?;
    Ok(Json(result))
}
