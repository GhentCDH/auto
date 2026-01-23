use axum::{extract::State, routing::get, Json, Router};
use serde::Deserialize;

use crate::service::search;
use crate::{AppState, Result};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(global_search))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
}

async fn global_search(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<impl axum::response::IntoResponse> {
    let result = search::global_search(&state.pool, &query.q).await?;
    Ok(Json(result))
}
