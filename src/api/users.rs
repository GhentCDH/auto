//! Admin-only user management: accounts, roles, setup links and reset requests.
//!
//! Mounted under `/api/admin` behind the `require_admin` layer, so every handler
//! here can assume the caller is an admin.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::{AuthUser, Role};
use crate::service::user as user_service;
use crate::service::user::{ResetRequestSummary, UserSummary};
use crate::{AppState, Error, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", axum::routing::delete(delete_user))
        .route("/users/{id}/role", put(update_role))
        .route("/users/{id}/setup-link", post(create_setup_link))
        .route("/users/{id}/revoke-sessions", post(revoke_sessions))
        .route("/reset-requests", get(list_reset_requests))
}

#[utoipa::path(
    get,
    path = "/api/admin/users",
    tag = "admin",
    responses((status = 200, description = "All users", body = [UserSummary]))
)]
async fn list_users(State(state): State<AppState>) -> Result<impl IntoResponse> {
    Ok(Json(user_service::list_users(&state.pool).await?))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserBody {
    pub username: String,
    pub email: Option<String>,
    pub role: Role,
}

#[utoipa::path(
    post,
    path = "/api/admin/users",
    tag = "admin",
    request_body = CreateUserBody,
    responses(
        (status = 201, description = "User created", body = UserSummary),
        (status = 409, description = "Username already exists")
    )
)]
async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserBody>,
) -> Result<impl IntoResponse> {
    let id = user_service::create_user(
        &state.pool,
        &body.username,
        body.email.as_deref(),
        body.role,
    )
    .await?;
    let users = user_service::list_users(&state.pool).await?;
    let created = users
        .into_iter()
        .find(|u| u.id == id)
        .ok_or_else(|| Error::InternalError("created user not found".to_string()))?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    delete,
    path = "/api/admin/users/{id}",
    tag = "admin",
    params(("id" = String, Path, description = "User id")),
    responses(
        (status = 204, description = "User deleted"),
        (status = 400, description = "Cannot delete your own account")
    )
)]
async fn delete_user(
    State(state): State<AppState>,
    admin: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    if id == admin.id {
        return Err(Error::ValidationError(
            "you cannot delete your own account".to_string(),
        ));
    }
    user_service::delete_user(&state.pool, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRoleBody {
    pub role: Role,
}

#[utoipa::path(
    put,
    path = "/api/admin/users/{id}/role",
    tag = "admin",
    params(("id" = String, Path, description = "User id")),
    request_body = UpdateRoleBody,
    responses(
        (status = 204, description = "Role updated"),
        (status = 400, description = "Cannot change your own role")
    )
)]
async fn update_role(
    State(state): State<AppState>,
    admin: AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdateRoleBody>,
) -> Result<impl IntoResponse> {
    if id == admin.id {
        return Err(Error::ValidationError(
            "you cannot change your own role".to_string(),
        ));
    }
    user_service::update_user_role(&state.pool, &id, body.role).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SetupLink {
    pub token: String,
    /// Full link to share with the user.
    pub url: String,
}

#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/setup-link",
    tag = "admin",
    params(("id" = String, Path, description = "User id")),
    responses((status = 200, description = "Setup link", body = SetupLink))
)]
async fn create_setup_link(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let token = user_service::create_setup_token(&state.pool, &id).await?;
    let base = state.config.base_url.trim_end_matches('/');
    let url = format!("{base}/set-password/{token}");
    Ok(Json(SetupLink { token, url }))
}

#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/revoke-sessions",
    tag = "admin",
    params(("id" = String, Path, description = "User id")),
    responses((status = 204, description = "Sessions revoked"))
)]
async fn revoke_sessions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    user_service::revoke_sessions(&state.pool, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/admin/reset-requests",
    tag = "admin",
    responses((status = 200, description = "Pending reset requests", body = [ResetRequestSummary]))
)]
async fn list_reset_requests(State(state): State<AppState>) -> Result<impl IntoResponse> {
    Ok(Json(user_service::list_reset_requests(&state.pool).await?))
}
