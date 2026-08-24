use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use utoipa::ToSchema;

/// Healthcheck entity - HTTP checks for monitoring endpoints
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Healthcheck {
    pub id: String,
    pub name: String,
    pub application_id: Option<String>,
    pub service_id: Option<String>,
    pub kuma_id: Option<i32>,
    pub domain_id: String,
    pub protocol: String,
    pub path: String,
    pub method: String,
    pub headers: Option<String>,
    pub expected_status: i32,
    pub expected_body: Option<String>,
    pub timeout_seconds: i32,
    pub interval: i32,
    pub is_enabled: bool,
    pub notes: Option<String>,
    pub retry: i32,
    pub retry_interval: i32,
    pub request_body_encoding: String,
    pub request_body: Option<String>,
    pub http_auth_user: Option<String>,
    pub http_auth_pass: Option<String>,
    pub kuma_dirty: bool,
    pub notifications: bool,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new healthcheck
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateHealthcheck {
    #[serde(deserialize_with = "super::trim_str")]
    pub name: String,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub application_id: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub service_id: Option<String>,
    pub kuma_id: Option<i32>,
    #[serde(deserialize_with = "super::trim_str")]
    pub domain_id: String,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub protocol: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub path: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub method: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub headers: Option<String>,
    pub expected_status: Option<i32>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub expected_body: Option<String>,
    pub timeout_seconds: Option<i32>,
    pub interval: Option<i32>,
    pub is_enabled: Option<bool>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub notes: Option<String>,
    pub retry: Option<i32>,
    pub retry_interval: Option<i32>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub request_body_encoding: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub request_body: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub http_auth_user: Option<String>,
    pub http_auth_pass: Option<String>,
    pub notifications: Option<bool>,
}

impl CreateHealthcheck {
    /// Fill omitted defaultable fields from configured defaults.
    pub fn apply_defaults(&mut self, d: &crate::config::HealthcheckDefaults) {
        self.protocol.get_or_insert_with(|| d.protocol.clone());
        self.path.get_or_insert_with(|| d.path.clone());
        self.method.get_or_insert_with(|| d.method.clone());
        self.expected_status.get_or_insert(d.expected_status);
        self.timeout_seconds.get_or_insert(d.timeout_seconds);
        self.interval.get_or_insert(d.interval);
        self.is_enabled.get_or_insert(d.is_enabled);
        self.retry.get_or_insert(d.retry);
        self.retry_interval.get_or_insert(d.retry_interval);
        self.request_body_encoding
            .get_or_insert_with(|| d.request_body_encoding.clone());
        self.notifications.get_or_insert(d.notifications);
    }
}

/// DTO for updating a healthcheck
#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct UpdateHealthcheck {
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub application_id: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub service_id: Option<String>,
    pub kuma_id: Option<i32>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub domain_id: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub protocol: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub path: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub method: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub headers: Option<String>,
    pub expected_status: Option<i32>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub expected_body: Option<String>,
    pub timeout_seconds: Option<i32>,
    pub interval: Option<i32>,
    pub is_enabled: Option<bool>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub notes: Option<String>,
    pub retry: Option<i32>,
    pub retry_interval: Option<i32>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub request_body_encoding: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub request_body: Option<String>,
    #[serde(default, deserialize_with = "super::trim_opt_str")]
    pub http_auth_user: Option<String>,
    pub http_auth_pass: Option<String>,
    pub notifications: Option<bool>,
}

/// Healthcheck with resolved relations
#[derive(Debug, Serialize, ToSchema)]
pub struct KumaEndpoint {
    pub url: String,
}

/// Healthcheck with resolved relations
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthcheckWithRelations {
    #[serde(flatten)]
    pub healthcheck: Healthcheck,
    pub application_name: Option<String>,
    pub service_name: Option<String>,
    pub domain_fqdn: String,
    pub parsed_headers: Option<HashMap<String, String>>,
}

/// Lightweight healthcheck relation for embedding in Application/Service detail views
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct HealthcheckRelation {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub domain_fqdn: String,
    pub path: String,
    pub expected_status: i32,
    pub is_enabled: bool,
    pub kuma_id: Option<i32>,
    pub kuma_dirty: bool,
    pub notifications: bool,
}

/// Result of executing a healthcheck
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthcheckExecuteResult {
    pub healthcheck_id: String,
    pub url: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_time_ms: u64,
    pub body_match: Option<bool>,
    pub error: Option<String>,
    pub executed_at: String,
}

/// Kuma monitor export format
#[derive(Debug, Serialize, ToSchema)]
pub struct KumaMonitor {
    pub name: String,
    pub url: String,
    pub method: String,
    pub expected_status: i32,
    pub timeout: i32,
    pub headers: Option<HashMap<String, String>>,
    pub target_type: String,
    pub target_name: String,
}

impl HealthcheckWithRelations {
    pub fn url(&self) -> String {
        format!(
            "{}://{}{}",
            self.healthcheck.protocol, self.domain_fqdn, self.healthcheck.path
        )
    }
}
