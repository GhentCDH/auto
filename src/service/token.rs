//! Personal API tokens for non-browser clients.
//!
//! Tokens are high-entropy random strings; only their SHA-256 hash is stored.
//! A token authenticates as its owning user with that user's current role.

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use crate::auth::{AuthMethod, AuthUser, Role, generate_token};
use crate::models::{ApiToken, CreatedApiToken, new_id};
use crate::{Error, Result};

/// Hex-encoded SHA-256 of a token. Only hashes are persisted.
fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Create a token for `user_id` and return it with the plaintext secret.
pub async fn create(
    pool: &SqlitePool,
    user_id: &str,
    name: &str,
    expires_in_days: Option<i64>,
) -> Result<CreatedApiToken> {
    let id = new_id();
    let secret = format!("auto_{}", generate_token());
    let now = Utc::now().to_rfc3339();
    let expires_at = expires_in_days.map(|days| (Utc::now() + Duration::days(days)).to_rfc3339());

    sqlx::query(
        "INSERT INTO api_tokens (id, user_id, name, token_hash, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(name)
    .bind(hash_token(&secret))
    .bind(&now)
    .bind(&expires_at)
    .execute(pool)
    .await?;

    Ok(CreatedApiToken {
        token: ApiToken {
            id,
            name: name.to_string(),
            created_at: now,
            expires_at,
            last_used_at: None,
        },
        secret,
    })
}

/// List a user's tokens, newest first.
pub async fn list(pool: &SqlitePool, user_id: &str) -> Result<Vec<ApiToken>> {
    let tokens = sqlx::query_as::<_, ApiToken>(
        "SELECT id, name, created_at, expires_at, last_used_at
         FROM api_tokens
         WHERE user_id = ?1
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(tokens)
}

/// Revoke one of a user's tokens.
pub async fn revoke(pool: &SqlitePool, user_id: &str, token_id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM api_tokens WHERE id = ?1 AND user_id = ?2")
        .bind(token_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(Error::NotFound(format!("token '{token_id}' not found")));
    }
    Ok(())
}

#[derive(sqlx::FromRow)]
struct TokenUserRow {
    token_id: String,
    expires_at: Option<String>,
    id: String,
    username: String,
    email: Option<String>,
    role: String,
}

/// Resolve a bearer token to its user, or `None` when unknown or expired.
pub async fn find_user(pool: &SqlitePool, secret: &str) -> Result<Option<AuthUser>> {
    let row = sqlx::query_as::<_, TokenUserRow>(
        "SELECT t.id AS token_id, t.expires_at, u.id, u.username, u.email, r.role
         FROM api_tokens t
         JOIN users u ON u.id = t.user_id
         JOIN user_roles r ON r.user_id = u.id
         WHERE t.token_hash = ?1",
    )
    .bind(hash_token(secret))
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else { return Ok(None) };
    if let Some(expires_at) = &row.expires_at
        && expires_at < &Utc::now().to_rfc3339()
    {
        return Ok(None);
    }

    // Best-effort usage stamp, throttled so busy clients don't write per request.
    let stale_before = (Utc::now() - Duration::minutes(5)).to_rfc3339();
    let _ = sqlx::query(
        "UPDATE api_tokens SET last_used_at = ?1
         WHERE id = ?2 AND (last_used_at IS NULL OR last_used_at < ?3)",
    )
    .bind(Utc::now().to_rfc3339())
    .bind(&row.token_id)
    .bind(&stale_before)
    .execute(pool)
    .await;

    Ok(Some(AuthUser {
        id: row.id,
        username: row.username,
        email: row.email,
        role: Role::parse(&row.role)?,
        method: AuthMethod::Token,
    }))
}
