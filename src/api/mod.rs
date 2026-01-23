use axum::extract::{Query, Request};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Form, Json, Router};

use crate::AppState;

pub fn api_routes(state: AppState) -> Router<AppState> {
    axum::Router::new()
        .route("/health", get(healthcheck))
        .with_state(state)
}

async fn healthcheck() -> &'static str {
    "ok"
}

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
