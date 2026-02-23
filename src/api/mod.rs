use axum::extract::{Path, Query, Request, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Form, Json, Router};

use crate::AppState;

pub mod applications;
pub mod dashboard;
pub mod domains;
pub mod healthchecks;
pub mod infra;
pub mod notes;
pub mod people;
pub mod search;
pub mod services;
pub mod shares;
pub mod stacks;

pub fn api_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/health", get(healthcheck))
        .route("/version", get(version))
        .nest("/applications", applications::routes())
        .nest("/services", services::routes())
        .nest("/infra", infra::routes())
        .nest("/domains", domains::routes())
        .nest("/people", people::routes())
        .nest("/shares", shares::routes())
        .nest("/notes", notes::routes())
        .nest("/stacks", stacks::routes())
        .nest("/healthchecks", healthchecks::routes())
        .nest("/dashboard", dashboard::routes())
        .nest("/search", search::routes())
        .route("/resolve/{id}", get(resolve_id))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = String)
    )
)]
async fn healthcheck() -> &'static str {
    "ok"
}

#[utoipa::path(
    get,
    path = "/api/version",
    tag = "health",
    responses(
        (status = 200, description = "API version information", body = serde_json::Value)
    )
)]
async fn version() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[utoipa::path(
    get,
    path = "/api/resolve/{id}",
    tag = "search",
    params(
        ("id" = String, Path, description = "Entity UUID to resolve")
    ),
    responses(
        (status = 200, description = "Resolved entity", body = crate::service::search::ResolvedEntity),
        (status = 404, description = "No entity found with this ID"),
        (status = 500, description = "Internal server error")
    )
)]
async fn resolve_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> crate::Result<impl IntoResponse> {
    let resolved = crate::service::search::resolve_id(&state.pool, &id).await?;
    Ok(Json(resolved))
}

#[allow(unused)]
pub struct FlexibleInput<T>(pub T);

impl<S, T> axum::extract::FromRequest<S> for FlexibleInput<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok());

        if let Some(ct) = content_type {
            if ct.contains("application/json") {
                let Json(data) = Json::<T>::from_request(req, state)
                    .await
                    .map_err(IntoResponse::into_response)?;
                return Ok(Self(data));
            }
            if ct.contains("application/x-www-form-urlencoded") {
                let Form(data) = Form::<T>::from_request(req, state)
                    .await
                    .map_err(IntoResponse::into_response)?;
                return Ok(Self(data));
            }
        }

        let Query(data) = Query::<T>::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;
        Ok(Self(data))
    }
}
