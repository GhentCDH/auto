//! Request middleware: session lookup and role enforcement.
//!
//! `auth_middleware` resolves the session cookie into an [`AuthUser`] and places
//! it on request extensions. `require_role` / `require_admin` are layered on top
//! of specific route groups; they assume `auth_middleware` ran first.

use axum::extract::{Request, State};
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::CookieJar;

use super::AuthUser;
use super::session::{self, SESSION_COOKIE};
use crate::{AppState, Error, Result};

/// Resolve the session cookie into an [`AuthUser`] or reject with 401.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    let jar = CookieJar::from_headers(req.headers());
    let token = jar
        .get(SESSION_COOKIE)
        .map(|c| c.value().to_string())
        .ok_or_else(|| Error::Unauthorized("authentication required".to_string()))?;

    let user = session::load_session(&state.pool, &token)
        .await?
        .ok_or_else(|| Error::Unauthorized("invalid or expired session".to_string()))?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
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
