use axum::{Json, Router, extract::State, routing::get};
use serde::Deserialize;

use crate::service::search::{self, SearchResults};
use crate::{AppState, Result};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(global_search))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
}

#[utoipa::path(
    get,
    path = "/api/search",
    tag = "search",
    params(
        ("q" = String, Query, description = "Search query")
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResults),
        (status = 500, description = "Internal server error")
    )
)]
async fn global_search(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<impl axum::response::IntoResponse> {
    let result = search::global_search(&state.pool, &query.q).await?;
    Ok(Json(result))
}
