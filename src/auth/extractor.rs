//! Axum extractor that pulls the [`AuthUser`] placed on request extensions by
//! the session middleware. A handler taking `user: AuthUser` is thereby
//! guaranteed an authenticated request.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use super::AuthUser;
use crate::Error;

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| Error::Unauthorized("authentication required".to_string()))
    }
}
