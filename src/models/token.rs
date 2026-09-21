use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// A stored API token as shown to its owner. The secret is never included here.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct ApiToken {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
}

/// DTO for creating an API token.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiToken {
    #[serde(deserialize_with = "super::trim_str")]
    pub name: String,
    /// Lifetime in days; omit for a token that never expires.
    #[serde(default)]
    pub expires_in_days: Option<i64>,
}

/// An API token together with its plaintext secret, returned only on creation.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreatedApiToken {
    #[serde(flatten)]
    pub token: ApiToken,
    pub secret: String,
}
