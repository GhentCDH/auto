//! Database operations for authentication users, credentials and roles.

use sqlx::SqlitePool;

use crate::auth::password::verify_password;
use crate::auth::{AuthMethod, AuthUser, Role};
use crate::{Error, Result};

#[derive(sqlx::FromRow)]
struct CredentialRow {
    id: String,
    username: String,
    email: Option<String>,
    role: String,
    password_hash: Option<String>,
}

/// Verify a username/password pair and return the resulting password-method
/// [`AuthUser`]. All failure modes return the same opaque error to avoid leaking
/// which usernames exist or whether a password has been set.
pub async fn verify_credentials(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<AuthUser> {
    let unauthorized = || Error::Unauthorized("invalid username or password".to_string());

    let row = sqlx::query_as::<_, CredentialRow>(
        "SELECT u.id, u.username, u.email, r.role, p.password_hash
         FROM users u
         JOIN user_roles r ON r.user_id = u.id
         LEFT JOIN password_credentials p ON p.user_id = u.id
         WHERE u.username = ?1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?
    .ok_or_else(unauthorized)?;

    let hash = row.password_hash.ok_or_else(unauthorized)?;
    if !verify_password(password, &hash) {
        return Err(unauthorized());
    }

    Ok(AuthUser {
        id: row.id,
        username: row.username,
        email: row.email,
        role: Role::parse(&row.role)?,
        method: AuthMethod::Password,
    })
}
