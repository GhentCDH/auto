//! Request middleware: session lookup and role enforcement.
//!
//! `auth_middleware` resolves the session cookie into an [`AuthUser`] and places
//! it on request extensions. `require_role` / `require_admin` are layered on top
//! of specific route groups; they assume `auth_middleware` ran first.

use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, header};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::CookieJar;

use super::AuthUser;
use super::session::{self, SESSION_COOKIE};
use crate::{AppState, Error, Result};

/// Resolve a session cookie or bearer token into an [`AuthUser`], or reject
/// with 401. Browsers carry the session cookie; non-browser clients (for example
/// the TUI) carry a personal API token as `Authorization: Bearer <token>`.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    let user = resolve_user(&state, req.headers())
        .await?
        .ok_or_else(|| Error::Unauthorized("authentication required".to_string()))?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

/// Try the session cookie first, then a bearer token.
async fn resolve_user(state: &AppState, headers: &HeaderMap) -> Result<Option<AuthUser>> {
    let jar = CookieJar::from_headers(headers);
    if let Some(cookie) = jar.get(SESSION_COOKIE)
        && let Some(user) = session::load_session(&state.pool, cookie.value()).await?
    {
        return Ok(Some(user));
    }

    if let Some(token) = bearer_token(headers)
        && let Some(user) = crate::service::token::find_user(&state.pool, &token).await?
    {
        return Ok(Some(user));
    }

    Ok(None)
}

/// Extract a `Authorization: Bearer <token>` value.
fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}

/// Allow reads for any authenticated role; require editor or admin for mutations.
pub async fn require_role(req: Request, next: Next) -> Result<Response> {
    let user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| Error::Unauthorized("authentication required".to_string()))?;

    let is_read = matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    if !is_read && !user.role.can_edit() {
        return Err(Error::Forbidden(
            "this action requires editor or admin".to_string(),
        ));
    }
    Ok(next.run(req).await)
}

/// Restrict a route group to admins (user management).
pub async fn require_admin(req: Request, next: Next) -> Result<Response> {
    let user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| Error::Unauthorized("authentication required".to_string()))?;

    if !user.role.is_admin() {
        return Err(Error::Forbidden("this action requires admin".to_string()));
    }
    Ok(next.run(req).await)
}
