use axum::extract::{Query, Request};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Form, Json, Router};

use crate::AppState;

mod applications;
mod dashboard;
mod domains;
mod infra;
mod notes;
mod people;
mod search;
mod services;
mod shares;
mod stacks;

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
        .nest("/dashboard", dashboard::routes())
        .nest("/search", search::routes())
        .with_state(state)
}

async fn healthcheck() -> &'static str {
    "ok"
}

async fn version() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION")
    }))
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
