use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use rust_embed::Embed;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::{AppState, api};

pub fn router(state: AppState) -> Router {
    axum::Router::new()
        .nest("/api", api::api_routes(state.clone()))
        .route("/", get(serve_frontend))
        .route("/{*path}", get(serve_frontend))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(state)
}

#[derive(Embed)]
#[folder = "frontend-dist"]
struct FrontendAssets;

async fn serve_frontend(path: Option<Path<String>>) -> Response {
    let path = path
        .map(|p| p.0)
        .unwrap_or_else(|| "index.html".to_string());

    match FrontendAssets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        // fallback to index.html for Vue router
        None => match FrontendAssets::get("index.html") {
            Some(content) => Html(content.data).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
    }
}
