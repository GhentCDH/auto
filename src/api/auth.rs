//! Authentication endpoints (login, logout, current user).
//!
//! Pre-login routes (`login`) are mounted publicly; `logout` and `me` require a
//! valid session but no particular role. Password-setup, reset and OIDC live in
//! the sibling modules but share this route group.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::auth::{AuthUser, session};
use crate::{AppState, Error, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginBody,
    responses(
        (status = 200, description = "Logged in", body = AuthUser),
        (status = 401, description = "Invalid credentials")
    )
)]
async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginBody>,
) -> Result<impl IntoResponse> {
    if !state.config.auth_password_enabled {
        return Err(Error::Forbidden("password login is disabled".to_string()));
    }
    let user =
        crate::service::user::verify_credentials(&state.pool, &body.username, &body.password)
            .await?;
    let token = session::create_session(&state.pool, &user, state.config.session_hours).await?;
    let secure = state.config.base_url.starts_with("https");
    let jar = session::set_session_cookie(jar, token, state.config.session_hours, secure);
    Ok((jar, Json(user)))
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "auth",
    responses((status = 204, description = "Logged out"))
)]
async fn logout(State(state): State<AppState>, jar: CookieJar) -> Result<impl IntoResponse> {
    if let Some(cookie) = jar.get(session::SESSION_COOKIE) {
        session::delete_session(&state.pool, cookie.value()).await?;
    }
    let jar = session::clear_session_cookie(jar);
    Ok((jar, StatusCode::NO_CONTENT))
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current user", body = AuthUser),
        (status = 401, description = "Not authenticated")
    )
)]
async fn me(user: AuthUser) -> Json<AuthUser> {
    Json(user)
}
