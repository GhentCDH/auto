//! Browser-assisted (loopback) login for non-browser clients.
//!
//! The TUI opens a local listener and asks for a flow bound to its port. The
//! user logs in in the browser (password or OIDC); once a session exists the
//! browser is redirected to the loopback listener with a one-time `code`, which
//! the client exchanges for a normal session token. No API token is involved.

use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use crate::auth::generate_token;
use crate::{Error, Result};

/// How long a started flow stays valid before the browser must redo the login.
const FLOW_TTL_MINUTES: i64 = 10;

/// A flow that has been authorized by a logged-in browser session.
pub struct Authorized {
    pub code: String,
    pub port: u16,
}

/// The identity bound to a successful code exchange.
pub struct Exchanged {
    pub user_id: String,
    pub method: String,
}

/// Start a flow for a client listening on `port`; returns the flow id.
pub async fn start(pool: &SqlitePool, port: u16) -> Result<String> {
    // Opportunistically drop expired flows; they are useless either way.
    sqlx::query("DELETE FROM cli_login_flows WHERE expires_at <= ?1")
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await?;

    let id = generate_token();
    let expires_at = (Utc::now() + Duration::minutes(FLOW_TTL_MINUTES)).to_rfc3339();
    sqlx::query("INSERT INTO cli_login_flows (id, port, expires_at) VALUES (?1, ?2, ?3)")
        .bind(&id)
        .bind(port as i64)
        .bind(&expires_at)
        .execute(pool)
        .await?;
    Ok(id)
}

/// Bind a logged-in user to the flow and mint its one-time code. Returns `None`
/// when the flow is unknown, expired or already consumed.
pub async fn authorize(
    pool: &SqlitePool,
    flow_id: &str,
    user_id: &str,
    method: &str,
) -> Result<Option<Authorized>> {
    let code = generate_token();
    let result = sqlx::query(
        "UPDATE cli_login_flows SET user_id = ?1, method = ?2, code = ?3
         WHERE id = ?4 AND used = 0 AND expires_at > ?5",
    )
    .bind(user_id)
    .bind(method)
    .bind(&code)
    .bind(flow_id)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    let port = sqlx::query_scalar::<_, i64>("SELECT port FROM cli_login_flows WHERE id = ?1")
        .bind(flow_id)
        .fetch_one(pool)
        .await?;

    Ok(Some(Authorized {
        code,
        port: port as u16,
    }))
}

/// Consume a one-time code and return the bound identity.
pub async fn exchange(pool: &SqlitePool, flow_id: &str, code: &str) -> Result<Exchanged> {
    let result = sqlx::query(
        "UPDATE cli_login_flows SET used = 1
         WHERE id = ?1 AND code = ?2 AND used = 0 AND user_id IS NOT NULL AND expires_at > ?3",
    )
    .bind(flow_id)
    .bind(code)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(Error::Unauthorized(
            "invalid or expired login code".to_string(),
        ));
    }

    let row = sqlx::query_as::<_, (Option<String>, Option<String>)>(
        "SELECT user_id, method FROM cli_login_flows WHERE id = ?1",
    )
    .bind(flow_id)
    .fetch_one(pool)
    .await?;

    let user_id = row
        .0
        .ok_or_else(|| Error::InternalError("flow lost its user".to_string()))?;
    Ok(Exchanged {
        user_id,
        method: row.1.unwrap_or_else(|| "password".to_string()),
    })
}
