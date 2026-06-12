//! Server-side session storage and the session cookie helpers.

use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use super::{AuthMethod, AuthUser, Role, generate_token};
use crate::Result;

/// Name of the opaque session cookie.
pub const SESSION_COOKIE: &str = "auto_session";

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    user_id: String,
    username: String,
    email: Option<String>,
    role: String,
    method: String,
    expires_at: String,
}

/// Create a session for `user` and return its opaque token.
pub async fn create_session(
    pool: &SqlitePool,
    user: &AuthUser,
    session_hours: i64,
) -> Result<String> {
    let token = generate_token();
    let expires_at = (Utc::now() + Duration::hours(session_hours)).to_rfc3339();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, method, role, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&token)
    .bind(&user.id)
    .bind(user.method.to_db())
    .bind(user.role.as_str())
    .bind(&expires_at)
    .execute(pool)
    .await?;
    Ok(token)
}

/// Look up a session token and rebuild the [`AuthUser`]. Returns `None` when the
/// token is unknown or expired (expired rows are deleted opportunistically).
pub async fn load_session(pool: &SqlitePool, token: &str) -> Result<Option<AuthUser>> {
    let row = sqlx::query_as::<_, SessionRow>(
        "SELECT s.id, s.user_id, u.username, u.email, s.role, s.method, s.expires_at
         FROM sessions s JOIN users u ON u.id = s.user_id
         WHERE s.id = ?1",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else { return Ok(None) };

    if row.expires_at < Utc::now().to_rfc3339() {
        delete_session(pool, &row.id).await?;
        return Ok(None);
    }

    Ok(Some(AuthUser {
        id: row.user_id,
        username: row.username,
        email: row.email,
        role: Role::parse(&row.role)?,
        method: AuthMethod::from_db(&row.method),
    }))
}

pub async fn delete_session(pool: &SqlitePool, token: &str) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE id = ?1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

/// Revoke every active session for a user (admin action or role change).
pub async fn delete_user_sessions(pool: &SqlitePool, user_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE user_id = ?1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Update the denormalised role on a user's live sessions after a role change,
/// so the change takes effect without forcing a re-login.
pub async fn update_user_sessions_role(pool: &SqlitePool, user_id: &str, role: Role) -> Result<()> {
    sqlx::query("UPDATE sessions SET role = ?1 WHERE user_id = ?2")
        .bind(role.as_str())
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Attach the session cookie. `secure` should be true behind HTTPS.
pub fn set_session_cookie(jar: CookieJar, token: String, hours: i64, secure: bool) -> CookieJar {
    let cookie = Cookie::build((SESSION_COOKIE, token))
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time::Duration::hours(hours))
        .build();
    jar.add(cookie)
}

/// Clear the session cookie.
pub fn clear_session_cookie(jar: CookieJar) -> CookieJar {
    jar.remove(Cookie::build(SESSION_COOKIE).path("/").build())
}
