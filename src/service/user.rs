//! Database operations for authentication users, credentials and roles.

use chrono::Utc;
use sqlx::SqlitePool;

use crate::auth::password::{hash_password, verify_password};
use crate::auth::{AuthMethod, AuthUser, Role, generate_token};
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

/// Consume a setup token to set a user's password. Handles both first-login
/// setup and admin-fulfilled resets: it marks the token used and resolves any
/// pending reset request for that user.
pub async fn set_password_with_token(
    pool: &SqlitePool,
    token: &str,
    new_password: &str,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let user_id = sqlx::query_scalar::<_, String>(
        "SELECT user_id FROM password_setup_tokens
         WHERE token = ?1 AND used = 0 AND expires_at > ?2",
    )
    .bind(token)
    .bind(&now)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::ValidationError("invalid or expired setup link".to_string()))?;

    let hash = hash_password(new_password)?;
    let mut tx = pool.begin().await?;

    sqlx::query(
        "INSERT INTO password_credentials (user_id, password_hash) VALUES (?1, ?2)
         ON CONFLICT(user_id) DO UPDATE SET password_hash = excluded.password_hash,
                                            updated_at = datetime('now')",
    )
    .bind(&user_id)
    .bind(&hash)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE password_setup_tokens SET used = 1 WHERE token = ?1")
        .bind(token)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "UPDATE password_reset_requests SET status = 'fulfilled'
         WHERE user_id = ?1 AND status = 'pending'",
    )
    .bind(&user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Change a logged-in user's password after verifying their current one.
pub async fn change_password(
    pool: &SqlitePool,
    user_id: &str,
    current_password: &str,
    new_password: &str,
) -> Result<()> {
    let hash = sqlx::query_scalar::<_, Option<String>>(
        "SELECT password_hash FROM password_credentials WHERE user_id = ?1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .flatten()
    .ok_or_else(|| Error::ValidationError("no password set for this account".to_string()))?;

    if !verify_password(current_password, &hash) {
        return Err(Error::Unauthorized(
            "current password is incorrect".to_string(),
        ));
    }

    let new_hash = hash_password(new_password)?;
    sqlx::query(
        "UPDATE password_credentials SET password_hash = ?1, updated_at = datetime('now')
         WHERE user_id = ?2",
    )
    .bind(&new_hash)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Record a forgotten-password request for `username`. Silently does nothing if
/// the username is unknown or a request is already pending, so the endpoint can
/// always respond identically and avoid leaking which usernames exist.
pub async fn create_reset_request(pool: &SqlitePool, username: &str) -> Result<()> {
    let user_id = sqlx::query_scalar::<_, String>("SELECT id FROM users WHERE username = ?1")
        .bind(username)
        .fetch_optional(pool)
        .await?;

    let Some(user_id) = user_id else {
        return Ok(());
    };

    let pending = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM password_reset_requests WHERE user_id = ?1 AND status = 'pending'",
    )
    .bind(&user_id)
    .fetch_one(pool)
    .await?;
    if pending > 0 {
        return Ok(());
    }

    sqlx::query("INSERT INTO password_reset_requests (id, user_id) VALUES (?1, ?2)")
        .bind(generate_token())
        .bind(&user_id)
        .execute(pool)
        .await?;
    Ok(())
}
