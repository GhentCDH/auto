pub mod models;
pub mod sse;

use base64::Engine;
use color_eyre::eyre::{Result, eyre};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::config::ResolvedConfig;
use models::*;

/// The entity kinds browsable in the TUI, in tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityKind {
    Applications,
    Services,
    Infra,
    Domains,
    People,
    Shares,
    Stacks,
    Healthchecks,
}

impl EntityKind {
    pub const ALL: [EntityKind; 8] = [
        EntityKind::Applications,
        EntityKind::Services,
        EntityKind::Infra,
        EntityKind::Domains,
        EntityKind::People,
        EntityKind::Shares,
        EntityKind::Stacks,
        EntityKind::Healthchecks,
    ];

    /// URL path segment under `/api`.
    pub fn path(self) -> &'static str {
        match self {
            EntityKind::Applications => "applications",
            EntityKind::Services => "services",
            EntityKind::Infra => "infra",
            EntityKind::Domains => "domains",
            EntityKind::People => "people",
            EntityKind::Shares => "shares",
            EntityKind::Stacks => "stacks",
            EntityKind::Healthchecks => "healthchecks",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            EntityKind::Applications => "Applications",
            EntityKind::Services => "Services",
            EntityKind::Infra => "Infra",
            EntityKind::Domains => "Domains",
            EntityKind::People => "People",
            EntityKind::Shares => "Shares",
            EntityKind::Stacks => "Stacks",
            EntityKind::Healthchecks => "Healthchecks",
        }
    }

    /// Map the backend's `entity_type` discriminator to a kind.
    pub fn from_entity_type(entity_type: &str) -> Option<Self> {
        Some(match entity_type {
            "application" => EntityKind::Applications,
            "service" => EntityKind::Services,
            "infra" => EntityKind::Infra,
            "domain" => EntityKind::Domains,
            "person" => EntityKind::People,
            "network_share" => EntityKind::Shares,
            "stack" => EntityKind::Stacks,
            "healthcheck" => EntityKind::Healthchecks,
            _ => return None,
        })
    }
}

/// HTTP client for the auto API. Cheap to clone; carries the precomputed
/// basic-auth header for the reverse proxy in front of the backend.
#[derive(Debug, Clone)]
pub struct ApiClient {
    http: reqwest::Client,
    base: String,
    auth: Option<String>,
}

impl ApiClient {
    pub fn new(config: &ResolvedConfig) -> Result<Self> {
        let auth = config.credentials.as_ref().map(|(user, pass)| {
            let encoded =
                base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"));
            format!("Basic {encoded}")
        });
        Ok(Self {
            http: reqwest::Client::builder().build()?,
            base: format!("{}/api", config.url),
            auth,
        })
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let mut builder = self.http.request(method, format!("{}{path}", self.base));
        if let Some(auth) = &self.auth {
            builder = builder.header(reqwest::header::AUTHORIZATION, auth.clone());
        }
        builder
    }

    async fn send<T: DeserializeOwned>(&self, builder: reqwest::RequestBuilder) -> Result<T> {
        let response = builder.send().await?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(eyre!(
                "authentication failed ({status}): check basic-auth credentials for the reverse proxy"
            ));
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(eyre!("request failed ({status}): {body}"));
        }
        Ok(response.json().await?)
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T> {
        self.send(self.request(reqwest::Method::GET, path).query(query))
            .await
    }

    /// POST returning no useful body (backend replies 204 or a JSON blob we
    /// don't need) — only the success/failure matters to the UI.
    async fn post_empty(&self, path: &str) -> Result<()> {
        let response = self.request(reqwest::Method::POST, path).send().await?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(eyre!(
                "authentication failed ({status}): check basic-auth credentials for the reverse proxy"
            ));
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(eyre!("request failed ({status}): {body}"));
        }
        Ok(())
    }

    /// Request builder for the SSE uptime stream (sent by the SSE task).
    fn uptime_stream_request(&self) -> reqwest::RequestBuilder {
        self.request(reqwest::Method::GET, "/healthchecks/uptime/stream")
            .header(reqwest::header::ACCEPT, "text/event-stream")
    }

    pub async fn dashboard(&self) -> Result<DashboardStats> {
        self.get_json("/dashboard/stats", &[]).await
    }

    /// Paginated entity list. Bodies stay untyped: every entity shares the
    /// same flattened `WithRelations` JSON shape and is rendered generically.
    pub async fn list(
        &self,
        entity: EntityKind,
        page: u32,
        per_page: u32,
        search: &str,
    ) -> Result<PaginatedResponse<Value>> {
        let mut query = vec![
            ("page", page.to_string()),
            ("per_page", per_page.to_string()),
        ];
        if !search.is_empty() {
            query.push(("search", search.to_string()));
        }
        self.get_json(&format!("/{}", entity.path()), &query).await
    }

    pub async fn detail(&self, entity: EntityKind, id: &str) -> Result<Value> {
        self.get_json(&format!("/{}/{id}", entity.path()), &[])
            .await
    }

    pub async fn search(&self, q: &str) -> Result<SearchResults> {
        self.get_json("/search", &[("q", q.to_string())]).await
    }

    pub async fn dns(&self, domain_id: &str) -> Result<DnsLookup> {
        self.get_json(&format!("/domains/{domain_id}/dns"), &[])
            .await
    }

    pub async fn execute_healthcheck(&self, id: &str) -> Result<HealthcheckExecuteResult> {
        self.get_json(&format!("/healthchecks/{id}/execute"), &[])
            .await
    }

    pub async fn infra_sync_all(&self) -> Result<()> {
        self.post_empty("/infra/sync").await
    }

    pub async fn infra_sync_one(&self, id: &str) -> Result<()> {
        self.post_empty(&format!("/infra/{id}/sync")).await
    }

    pub async fn kuma_sync_all(&self) -> Result<()> {
        self.post_empty("/healthchecks/sync/kuma").await
    }

    pub async fn kuma_sync_one(&self, id: &str) -> Result<()> {
        self.post_empty(&format!("/healthchecks/sync/kuma/{id}"))
            .await
    }
}
