use crate::error::Error as AutoError;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

/// Healthcheck entity - HTTP checks for monitoring endpoints
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
    pub is_enabled: bool,
    pub notes: Option<String>,
    pub retry: i32,
    pub retry_interval: i32,
    pub request_body_encoding: String,
    pub request_body: Option<String>,
    pub http_auth_user: Option<String>,
    pub http_auth_pass: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// DTO for creating a new healthcheck
#[derive(Debug, Deserialize)]
pub struct CreateHealthcheck {
    pub name: String,
    pub application_id: Option<String>,
    pub service_id: Option<String>,
    pub kuma_id: Option<i32>,
    pub domain_id: String,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default = "default_method")]
    pub method: String,
    pub headers: Option<String>,
    #[serde(default = "default_expected_status")]
    pub expected_status: i32,
    pub expected_body: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: i32,
    #[serde(default = "default_enabled")]
    pub is_enabled: bool,
    pub notes: Option<String>,
    #[serde(default)]
    pub retry: i32,
    #[serde(default = "default_retry_interval")]
    pub retry_interval: i32,
    #[serde(default = "default_request_body_encoding")]
    pub request_body_encoding: String,
    pub request_body: Option<String>,
    pub http_auth_user: Option<String>,
    pub http_auth_pass: Option<String>,
}

/// DTO for updating a healthcheck
#[derive(Debug, Deserialize)]
pub struct UpdateHealthcheck {
    pub name: Option<String>,
    pub application_id: Option<String>,
    pub service_id: Option<String>,
    pub kuma_id: Option<i32>,
    pub domain_id: Option<String>,
    pub protocol: Option<String>,
    pub path: Option<String>,
    pub method: Option<String>,
    pub headers: Option<String>,
    pub expected_status: Option<i32>,
    pub expected_body: Option<String>,
    pub timeout_seconds: Option<i32>,
    pub is_enabled: Option<bool>,
    pub notes: Option<String>,
    pub retry: Option<i32>,
    pub retry_interval: Option<i32>,
    pub request_body_encoding: Option<String>,
    pub request_body: Option<String>,
    pub http_auth_user: Option<String>,
    pub http_auth_pass: Option<String>,
}

fn default_protocol() -> String {
    "https".to_string()
}

fn default_path() -> String {
    "/".to_string()
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_expected_status() -> i32 {
    200
}

fn default_timeout() -> i32 {
    30
}

fn default_enabled() -> bool {
    true
}

fn default_retry_interval() -> i32 {
    60
}

fn default_request_body_encoding() -> String {
    "JSON".to_string()
}

/// Healthcheck with resolved relations
#[derive(Debug, Serialize)]
pub struct HealthcheckWithRelations {
    #[serde(flatten)]
    pub healthcheck: Healthcheck,
    pub application_name: Option<String>,
    pub service_name: Option<String>,
    pub domain_fqdn: String,
    pub parsed_headers: Option<HashMap<String, String>>,
}

/// Lightweight healthcheck relation for embedding in Application/Service detail views
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HealthcheckRelation {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub domain_fqdn: String,
    pub path: String,
    pub expected_status: i32,
    pub is_enabled: bool,
    pub kuma_id: Option<i32>,
}

/// Result of executing a healthcheck
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
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
