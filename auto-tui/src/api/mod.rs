pub mod models;
pub mod sse;

use std::sync::{Arc, RwLock};

use color_eyre::eyre::{Result, eyre};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::config::ResolvedConfig;
use crate::session_store;
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

/// Outcome of the silent (no user interaction) authentication attempt.
#[derive(Debug)]
pub enum AuthStatus {
    Authenticated,
    LoginRequired { reason: Option<String> },
}

#[derive(Debug, Deserialize)]
struct CliStartResponse {
    flow_id: String,
    authorize_url: String,
}

#[derive(Debug, Deserialize)]
struct CliExchangeResponse {
    token: String,
}

/// A started browser-loopback login: the URL to open and the listener awaiting
/// the callback.
pub struct LoopbackSession {
    pub authorize_url: String,
    flow_id: String,
    listener: tokio::net::TcpListener,
}

/// HTTP client for the auto API. Cheap to clone; carries an optional bearer
/// token and/or session cookie. The session is shared so a login performed
/// after construction is visible to every clone.
#[derive(Debug, Clone)]
pub struct ApiClient {
    http: reqwest::Client,
    root: String,
    base: String,
    token: Option<String>,
    username: Option<String>,
    password: Option<String>,
    session: Arc<RwLock<Option<String>>>,
}

impl ApiClient {
    pub fn new(config: &ResolvedConfig) -> Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder().build()?,
            root: config.url.clone(),
            base: format!("{}/api", config.url),
            token: config.token.clone(),
            username: config.username.clone(),
            password: config.password.clone(),
            session: Arc::new(RwLock::new(None)),
        })
    }

    /// Configured username, used to prefill the login screen.
    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    /// Configured password, used to prefill the login screen.
    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    /// Base server URL (no `/api` suffix), e.g. for opening in a browser.
    pub fn server_url(&self) -> &str {
        &self.root
    }

    /// Set (or clear) the session token used as the `auto_session` cookie.
    pub fn set_session(&self, token: Option<String>) {
        if let Ok(mut guard) = self.session.write() {
            *guard = token;
        }
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let mut builder = self.http.request(method, format!("{}{path}", self.base));
        if let Some(token) = &self.token {
            builder = builder.bearer_auth(token);
        }
        let session = self.session.read().ok().and_then(|guard| guard.clone());
        if let Some(session) = session {
            builder = builder.header(reqwest::header::COOKIE, format!("auto_session={session}"));
        }
        builder
    }

    /// Authenticate without user interaction: open mode, a configured API
    /// token, or a previously stored loopback session.
    pub async fn try_silent_auth(&self) -> Result<AuthStatus> {
        let config: ConfigResponse = self.get_json("/config", &[]).await?;
        if !config.auth.password_enabled && !config.auth.oidc_enabled {
            return Ok(AuthStatus::Authenticated);
        }

        if self.token.is_some() {
            if self.check_me().await {
                return Ok(AuthStatus::Authenticated);
            }
            return Ok(AuthStatus::LoginRequired {
                reason: Some("the configured API token was rejected".to_string()),
            });
        }

        if let Some(session) = session_store::load() {
            self.set_session(Some(session));
            if self.check_me().await {
                return Ok(AuthStatus::Authenticated);
            }
            self.set_session(None);
            session_store::clear();
        }

        Ok(AuthStatus::LoginRequired { reason: None })
    }

    /// Whether the current token/session is accepted by `GET /auth/me`.
    async fn check_me(&self) -> bool {
        match self.request(reqwest::Method::GET, "/auth/me").send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Password login; stores the returned session cookie and persists it.
    pub async fn login_password(&self, username: &str, password: &str) -> Result<()> {
        let response = self
            .http
            .post(format!("{}/auth/login", self.base))
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(eyre!("login failed ({status}): {body}"));
        }
        let token = session_cookie(response.headers())
            .ok_or_else(|| eyre!("login succeeded but no session cookie was returned"))?;
        self.set_session(Some(token.clone()));
        session_store::save(&token)?;
        Ok(())
    }

    /// Start a browser-loopback login and return the session awaiting its
    /// callback. The caller opens [`LoopbackSession::authorize_url`].
    pub async fn start_loopback(&self) -> Result<LoopbackSession> {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let port = listener.local_addr()?.port();
        let start: CliStartResponse = self
            .post_json("/auth/cli/start", &serde_json::json!({ "port": port }))
            .await?;
        Ok(LoopbackSession {
            authorize_url: format!("{}{}", self.root, start.authorize_url),
            flow_id: start.flow_id,
            listener,
        })
    }

    /// POST a JSON body and deserialize the response.
    async fn post_json<T: DeserializeOwned>(&self, path: &str, body: &Value) -> Result<T> {
        let response = self
            .request(reqwest::Method::POST, path)
            .json(body)
            .send()
            .await?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(eyre!("authentication failed ({status})"));
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(eyre!("request failed ({status}): {body}"));
        }
        Ok(response.json().await?)
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

impl LoopbackSession {
    /// Wait for the browser callback, exchange the one-time code for a session,
    /// store it, and return it.
    pub async fn finish(self, client: &ApiClient) -> Result<String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let (mut socket, _) =
            tokio::time::timeout(std::time::Duration::from_secs(300), self.listener.accept())
                .await
                .map_err(|_| eyre!("timed out waiting for the browser"))??;

        let mut buffer = vec![0u8; 8192];
        let read = socket.read(&mut buffer).await?;
        let request = String::from_utf8_lossy(&buffer[..read]);
        let target = request.split_whitespace().nth(1).unwrap_or_default();
        let query = target
            .split_once('?')
            .map(|(_, query)| query)
            .unwrap_or_default();

        let mut flow = None;
        let mut code = None;
        for pair in query.split('&') {
            match pair.split_once('=') {
                Some(("flow", value)) => flow = Some(value.to_string()),
                Some(("code", value)) => code = Some(value.to_string()),
                _ => {}
            }
        }

        let body = "<!doctype html><meta charset=\"utf-8\"><title>Signed in</title>\
                    <body style=\"font-family:sans-serif;padding:2rem\">\
                    <h1>Signed in</h1><p>You can return to the terminal.</p></body>";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.flush().await;

        if flow.as_deref() != Some(self.flow_id.as_str()) {
            return Err(eyre!("login flow mismatch"));
        }
        let code = code.ok_or_else(|| eyre!("missing login code"))?;

        let exchange: CliExchangeResponse = client
            .post_json(
                "/auth/cli/exchange",
                &serde_json::json!({ "flow_id": self.flow_id, "code": code }),
            )
            .await?;
        client.set_session(Some(exchange.token.clone()));
        session_store::save(&exchange.token)?;
        Ok(exchange.token)
    }
}

/// Pull the `auto_session` value out of a response's `Set-Cookie` headers.
fn session_cookie(headers: &reqwest::header::HeaderMap) -> Option<String> {
    headers
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.split(';').next())
        .find_map(|pair| pair.strip_prefix("auto_session=").map(str::to_string))
}
