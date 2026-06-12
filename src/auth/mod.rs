//! Authentication: internal identity, sessions and credentials.
//!
//! The app normalises every login (password or OIDC) into an [`AuthUser`] and a
//! server-side session. After login the client only ever carries an opaque
//! session token, so handlers stay unaware of the original auth method.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod extractor;
pub mod password;
pub mod session;

/// Authorization role. Stored as a lowercase string in `user_roles` and `sessions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Editor,
    Viewer,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Editor => "editor",
            Role::Viewer => "viewer",
        }
    }

    pub fn parse(s: &str) -> crate::Result<Self> {
        match s {
            "admin" => Ok(Role::Admin),
            "editor" => Ok(Role::Editor),
            "viewer" => Ok(Role::Viewer),
            other => Err(crate::Error::InternalError(format!(
                "unknown role '{other}'"
            ))),
        }
    }

    /// Admins and editors may perform mutating operations.
    pub fn can_edit(self) -> bool {
        matches!(self, Role::Admin | Role::Editor)
    }

    pub fn is_admin(self) -> bool {
        matches!(self, Role::Admin)
    }
}

/// How a user authenticated. Serialised for `GET /api/auth/me`.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthMethod {
    Password,
    Oidc { issuer: String },
}

impl AuthMethod {
    /// Encode for the `sessions.method` column: `"password"` or `"oidc:<issuer>"`.
    pub fn to_db(&self) -> String {
        match self {
            AuthMethod::Password => "password".to_string(),
            AuthMethod::Oidc { issuer } => format!("oidc:{issuer}"),
        }
    }

    pub fn from_db(s: &str) -> Self {
        match s.strip_prefix("oidc:") {
            Some(issuer) => AuthMethod::Oidc {
                issuer: issuer.to_string(),
            },
            None => AuthMethod::Password,
        }
    }
}

/// The authenticated identity, independent of auth method. Set on request
/// extensions by the session middleware and pulled into handlers via the
/// [`extractor`] impl.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub role: Role,
    pub method: AuthMethod,
}

/// Generate an opaque 256-bit token, hex-encoded. Used for session ids and
/// password-setup / account-linking tokens.
pub fn generate_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
