//! Authentication endpoints (login, logout, current user).
//!
//! Pre-login routes (`login`) are mounted publicly; `logout` and `me` require a
//! valid session but no particular role. Password-setup, reset and OIDC live in
//! the sibling modules but share this route group.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use openidconnect::{Nonce, PkceCodeVerifier};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::{AuthMethod, AuthUser, session};
use crate::service::user as user_service;
use crate::{AppState, Error, Result};

/// Short-lived cookie holding the CSRF / nonce / PKCE values for an in-flight
/// OIDC login, replayed when the IdP redirects back to the callback.
const OIDC_FLOW_COOKIE: &str = "oidc_flow";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/set-password", post(set_password))
        .route("/change-password", post(change_password))
        .route("/reset-request", post(reset_request))
        .route("/oidc/start", get(oidc_start))
        .route("/oidc/callback", get(oidc_callback))
        .route("/link", post(link))
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetPasswordBody {
    pub token: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/set-password",
    tag = "auth",
    request_body = SetPasswordBody,
    responses(
        (status = 204, description = "Password set"),
        (status = 400, description = "Invalid or expired setup link")
    )
)]
async fn set_password(
    State(state): State<AppState>,
    Json(body): Json<SetPasswordBody>,
) -> Result<impl IntoResponse> {
    crate::service::user::set_password_with_token(&state.pool, &body.token, &body.password).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordBody {
    pub current_password: String,
    pub new_password: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/change-password",
    tag = "auth",
    request_body = ChangePasswordBody,
    responses(
        (status = 204, description = "Password changed"),
        (status = 401, description = "Current password incorrect")
    )
)]
async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChangePasswordBody>,
) -> Result<impl IntoResponse> {
    crate::service::user::change_password(
        &state.pool,
        &user.id,
        &body.current_password,
        &body.new_password,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetRequestBody {
    pub username: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/reset-request",
    tag = "auth",
    request_body = ResetRequestBody,
    responses((status = 204, description = "Request recorded if the account exists"))
)]
async fn reset_request(
    State(state): State<AppState>,
    Json(body): Json<ResetRequestBody>,
) -> Result<impl IntoResponse> {
    crate::service::user::create_reset_request(&state.pool, &body.username).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize, Deserialize)]
struct OidcFlow {
    csrf: String,
    nonce: String,
    pkce: String,
}

fn secure_from(state: &AppState) -> bool {
    state.config.base_url.starts_with("https")
}

/// Begin OIDC login: redirect to the IdP and stash the flow secrets in a cookie.
#[utoipa::path(
    get,
    path = "/api/auth/oidc/start",
    tag = "auth",
    responses((status = 303, description = "Redirect to the OIDC provider"))
)]
async fn oidc_start(State(state): State<AppState>, jar: CookieJar) -> Result<impl IntoResponse> {
    let provider = state
        .oidc_client
        .as_ref()
        .ok_or_else(|| Error::Forbidden("oidc login is disabled".to_string()))?;

    let (url, csrf, nonce, pkce) = provider.authorize_url();
    let flow = OidcFlow {
        csrf: csrf.secret().clone(),
        nonce: nonce.secret().clone(),
        pkce: pkce.secret().clone(),
    };
    let value = serde_json::to_string(&flow).map_err(|e| Error::InternalError(e.to_string()))?;
    let cookie = Cookie::build((OIDC_FLOW_COOKIE, value))
        .http_only(true)
        .secure(secure_from(&state))
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time::Duration::minutes(10))
        .build();

    Ok((jar.add(cookie), Redirect::to(&url)))
}

#[derive(Debug, Deserialize)]
struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

/// OIDC callback: validate the response, resolve or create the user, and either
/// log them in or redirect into the account-linking flow.
#[utoipa::path(
    get,
    path = "/api/auth/oidc/callback",
    tag = "auth",
    responses((status = 303, description = "Redirect to the app or the linking page"))
)]
async fn oidc_callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(params): Query<CallbackParams>,
) -> Result<impl IntoResponse> {
    let provider = state
        .oidc_client
        .as_ref()
        .ok_or_else(|| Error::Forbidden("oidc login is disabled".to_string()))?;

    if let Some(err) = params.error {
        return Err(Error::Unauthorized(format!("oidc provider error: {err}")));
    }
    let code = params
        .code
        .ok_or_else(|| Error::ValidationError("missing authorization code".to_string()))?;
    let returned_state = params
        .state
        .ok_or_else(|| Error::ValidationError("missing state".to_string()))?;

    let flow: OidcFlow = jar
        .get(OIDC_FLOW_COOKIE)
        .and_then(|c| serde_json::from_str(c.value()).ok())
        .ok_or_else(|| Error::ValidationError("missing or invalid oidc flow".to_string()))?;
    if returned_state != flow.csrf {
        return Err(Error::Unauthorized("oidc state mismatch".to_string()));
    }

    let identity = provider
        .exchange(
            code,
            PkceCodeVerifier::new(flow.pkce),
            &Nonce::new(flow.nonce),
        )
        .await?;

    // Done with the flow secrets regardless of outcome.
    let jar = jar.remove(Cookie::build(OIDC_FLOW_COOKIE).path("/").build());
    let secure = secure_from(&state);

    let user_id = match user_service::find_user_by_oidc(
        &state.pool,
        &identity.issuer,
        &identity.sub,
    )
    .await?
    {
        Some(id) => id,
        None => {
            // Unknown identity. In mixed-auth setups, an email matching an
            // existing account triggers the linking flow; otherwise we
            // auto-create a viewer account.
            let existing = match (&identity.email, state.config.auth_password_enabled) {
                (Some(email), true) => user_service::find_user_by_email(&state.pool, email).await?,
                _ => None,
            };
            match existing {
                Some(_) => {
                    let token =
                        user_service::store_pending_identity(&state.pool, &identity).await?;
                    return Ok((jar, Redirect::to(&format!("/link-account?token={token}"))));
                }
                None => user_service::create_oidc_user(&state.pool, &identity).await?,
            }
        }
    };

    let user = user_service::load_auth_user(
        &state.pool,
        &user_id,
        AuthMethod::Oidc {
            issuer: identity.issuer,
        },
    )
    .await?;
    let token = session::create_session(&state.pool, &user, state.config.session_hours).await?;
    let jar = session::set_session_cookie(jar, token, state.config.session_hours, secure);
    Ok((jar, Redirect::to("/")))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkBody {
    pub link_token: String,
    pub username: String,
    pub password: String,
}

/// Link a pending OIDC identity to an existing password account after proving
/// ownership of that account, then log in.
#[utoipa::path(
    post,
    path = "/api/auth/link",
    tag = "auth",
    request_body = LinkBody,
    responses(
        (status = 200, description = "Account linked and logged in", body = AuthUser),
        (status = 401, description = "Invalid credentials"),
        (status = 400, description = "Invalid or expired link token")
    )
)]
async fn link(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LinkBody>,
) -> Result<impl IntoResponse> {
    let identity = user_service::retrieve_pending_identity(&state.pool, &body.link_token).await?;
    let user =
        user_service::verify_credentials(&state.pool, &body.username, &body.password).await?;

    user_service::link_oidc_identity(&state.pool, &user.id, &identity.issuer, &identity.sub)
        .await?;
    user_service::delete_pending_identity(&state.pool, &body.link_token).await?;

    let authed = user_service::load_auth_user(
        &state.pool,
        &user.id,
        AuthMethod::Oidc {
            issuer: identity.issuer,
        },
    )
    .await?;
    let token = session::create_session(&state.pool, &authed, state.config.session_hours).await?;
    let jar =
        session::set_session_cookie(jar, token, state.config.session_hours, secure_from(&state));
    Ok((jar, Json(authed)))
}
