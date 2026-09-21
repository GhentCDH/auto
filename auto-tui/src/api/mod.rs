pub mod models;
pub mod sse;

use color_eyre::eyre::{Result, eyre};
use serde::Deserialize;
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
}

/// Public config served at `GET /api/config`; only what the client needs.
#[derive(Debug, Deserialize)]
struct ConfigResponse {
    auth: AuthConfig,
}

#[derive(Debug, Deserialize)]
struct AuthConfig {
    password_enabled: bool,
    oidc_enabled: bool,
}

/// HTTP client for the auto API. Cheap to clone; carries an optional bearer
/// token and a cookie store (populated by `POST /api/auth/login`).
#[derive(Debug, Clone)]
pub struct ApiClient {
    http: reqwest::Client,
    base: String,
    token: Option<String>,
    credentials: Option<(String, String)>,
}

impl ApiClient {
    pub fn new(config: &ResolvedConfig) -> Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder().cookie_store(true).build()?,
            base: format!("{}/api", config.url),
            token: config.token.clone(),
            credentials: config.credentials.clone(),
        })
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let mut builder = self.http.request(method, format!("{}{path}", self.base));
        if let Some(token) = &self.token {
            builder = builder.bearer_auth(token);
        }
        builder
    }

    /// Resolve authentication before the UI starts. In open mode (no login
    /// method enabled) this is a no-op; otherwise it validates the token or
    /// performs a password login so the cookie store is primed.
    pub async fn ensure_authenticated(&self) -> Result<()> {
        let config: ConfigResponse = self.get_json("/config", &[]).await?;
        if !config.auth.password_enabled && !config.auth.oidc_enabled {
            return Ok(());
        }

        if self.token.is_some() {
            let response = self
                .request(reqwest::Method::GET, "/auth/me")
                .send()
                .await?;
            return match response.status() {
                status if status.is_success() => Ok(()),
                reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                    Err(eyre!("API token rejected: check `token` / AUTO_TOKEN"))
                }
                status => Err(eyre!("authentication check failed ({status})")),
            };
        }

        if let Some((username, password)) = &self.credentials {
            if !config.auth.password_enabled {
                return Err(eyre!(
                    "this server offers OIDC login only; set `token` / AUTO_TOKEN to a \
                     personal API token created from the web UI"
                ));
            }
            let response = self
                .http
                .post(format!("{}/auth/login", self.base))
                .json(&serde_json::json!({ "username": username, "password": password }))
                .send()
                .await?;
            return match response.status() {
                status if status.is_success() => Ok(()),
                reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                    Err(eyre!("login failed: check username/password"))
                }
                status => Err(eyre!("login failed ({status})")),
            };
        }

        Err(eyre!(
            "this server requires authentication: set `username`/`password` or `token` \
             in ~/.config/auto-tui/config.toml (or AUTO_USERNAME/AUTO_PASSWORD/AUTO_TOKEN)"
        ))
    }

    async fn send<T: DeserializeOwned>(&self, builder: reqwest::RequestBuilder) -> Result<T> {
        let response = builder.send().await?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(eyre!(
                "authentication failed ({status}): check credentials or API token"
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
                "authentication failed ({status}): check credentials or API token"
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
