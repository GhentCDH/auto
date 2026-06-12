use axum::extract::{Path, Query, Request, State};
use axum::http::header;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Form, Json, Router};
use uuid::Uuid;

use crate::AppState;
use crate::auth::middleware::{auth_middleware, require_admin, require_role};

pub mod applications;
pub mod auth;
pub mod dashboard;
pub mod domains;
pub mod healthchecks;
pub mod infra;
pub mod notes;
pub mod outline;
pub mod people;
pub mod search;
pub mod services;
pub mod shares;
pub mod stacks;
pub mod users;

pub fn api_routes(state: AppState) -> Router<AppState> {
    // Public: no authentication. Pre-login, token-based self-service and config.
    let public = Router::new()
        .route("/health", get(healthcheck))
        .route("/version", get(version))
        .route("/config", get(config))
        .nest("/auth", auth::public_routes());

    // Authenticated, any role: session required but no role gate, so viewers can
    // read their own identity and change their own password.
    let session_only = Router::new()
        .nest("/auth", auth::session_routes())
        .route_layer(from_fn_with_state(state.clone(), auth_middleware));

    // Data routes: any role may read; only editor/admin may mutate.
    let data = Router::new()
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
        .nest("/outline", outline::routes())
        .route("/resolve/{id}", get(resolve_id))
        // auth_middleware is added last so it is outermost and runs before
        // require_role, which reads the AuthUser it inserts.
        .route_layer(from_fn(require_role))
        .route_layer(from_fn_with_state(state.clone(), auth_middleware));

    // Admin: user management, admins only.
    let admin = Router::new()
        .nest("/admin", users::routes())
        .route_layer(from_fn(require_admin))
        .route_layer(from_fn_with_state(state.clone(), auth_middleware));

    public
        .merge(session_only)
        .merge(data)
        .merge(admin)
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
    path = "/api/config",
    tag = "config",
    responses(
        (status = 200, description = "Configurable defaults and dropdown options", body = crate::config::PublicConfig)
    )
)]
async fn config(State(state): State<AppState>) -> Json<crate::config::PublicConfig> {
    Json(crate::config::PublicConfig::from(&state.config))
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

/// Resolve an id into a verified Uuid of particular entity type
async fn complete_id_of_type(
    state: &AppState,
    id: &str,
    entity_type: &str,
) -> crate::Result<String> {
    if id.parse::<Uuid>().is_err() {
        let resolved = crate::service::search::resolve_id(&state.pool, id).await?;
        if resolved.entity_type != entity_type {
            return Err(crate::Error::NotFound(format!(
                "No entity found with id {id}"
            )));
        }
        Ok(resolved.id)
    } else {
        Ok(id.to_string())
    }
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
