/*!
 * Custom Uptime Kuma client using Socket.IO.
 *
 * Replaces the `kuma-client` crate which panicked on float deserialization.
 * Only implements the subset needed: connect, login, edit/add monitors.
 *
 * The database of Auto is the single source of truth — this module
 * pushes healthcheck state to Kuma.
 */

use std::sync::Arc;
use std::time::Duration;

use futures::FutureExt as _;
use rust_socketio::{
    Payload,
    asynchronous::{Client as SocketClient, ClientBuilder},
};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::{AppState, Error, Result, models::HealthcheckWithRelations, service};

// ── KumaClient ──────────────────────────────────────────────────────

struct KumaClient {
    socket: SocketClient,
}

impl KumaClient {
    /// Connect to Uptime Kuma via Socket.IO and authenticate.
    ///
    /// `ClientBuilder::connect()` returns as soon as the Engine.IO transport
    /// is up, but the Socket.IO connection isn't fully ready until the server
    /// starts pushing events. We wait for the first server event (Kuma sends
    /// "info" immediately) before emitting anything, otherwise the ack
    /// callback is never fired.
    async fn connect(url: &url::Url, username: &str, password: &str) -> Result<Self> {
        let ready = Arc::new(tokio::sync::Notify::new());
        let ready_signal = ready.clone();

        let socket = ClientBuilder::new(url.as_str())
            .on_any(move |event, _payload, _client| {
                let ready = ready_signal.clone();
                async move {
                    debug!("Kuma event: {event}");
                    ready.notify_one();
                }
                .boxed()
            })
            .connect()
            .await
            .map_err(|e| Error::KumaError(format!("Connection failed: {e}")))?;

        // Wait until the server pushes its first event (usually "info"),
        // which confirms the Socket.IO connection is fully established.
        tokio::time::timeout(Duration::from_secs(10), ready.notified())
            .await
            .map_err(|_| Error::KumaError("Timed out waiting for Kuma to be ready".into()))?;

        debug!("Socket.IO connection established");

        let client = Self { socket };
        client.login(username, password).await?;
        Ok(client)
    }

    /// Emit a Socket.IO event and wait for the ack response.
    async fn call(&self, event: &str, payload: Value) -> Result<Value> {
        let (tx, mut rx) = mpsc::channel::<Value>(1);

        self.socket
            .emit_with_ack(
                event,
                Payload::Text(vec![payload]),
                Duration::from_secs(30),
                move |message: Payload, _| {
                    let tx = tx.clone();
                    async move {
                        if let Payload::Text(values) = message {
                            // Ack responses are nested: [[{ok: true, ...}]]
                            let json = Value::Array(values);
                            let response = json
                                .pointer("/0/0")
                                .or_else(|| json.pointer("/0"))
                                .cloned()
                                .unwrap_or(Value::Null);
                            let _ = tx.send(response).await;
                        }
                    }
                    .boxed()
                },
            )
            .await
            .map_err(|e| Error::KumaError(format!("Emit '{event}' failed: {e}")))?;

        rx.recv()
            .await
            .ok_or_else(|| Error::KumaError(format!("No ack response for '{event}'")))
    }

    async fn login(&self, username: &str, password: &str) -> Result<()> {
        debug!("Logging in on Kuma");
        let response = self
            .call(
                "login",
                json!({
                    "username": username,
                    "password": password,
                    "token": ""
                }),
            )
            .await?;

        debug!("Got response from login call");
        if response.get("ok").and_then(|v| v.as_bool()) != Some(true) {
            let msg = response
                .get("msg")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(Error::KumaError(format!("Login failed: {msg}")));
        }

        info!("Logged in to Kuma");
        Ok(())
    }

    async fn edit_monitor(&self, monitor: Value) -> Result<i32> {
        let response = self.call("editMonitor", monitor).await?;
        extract_monitor_id(&response, "editMonitor")
    }

    async fn add_monitor(&self, monitor: Value) -> Result<i32> {
        let response = self.call("add", monitor).await?;
        extract_monitor_id(&response, "add")
    }

    async fn disconnect(self) -> Result<()> {
        self.socket
            .disconnect()
            .await
            .map_err(|e| Error::KumaError(format!("Disconnect failed: {e}")))?;
        Ok(())
    }
}

fn extract_monitor_id(response: &Value, event: &str) -> Result<i32> {
    if response.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        let msg = response
            .get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        return Err(Error::KumaError(format!("{event} failed: {msg}")));
    }
    response
        .get("monitorID")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| Error::KumaError(format!("Missing monitorID in {event} response")))
}

// ── Monitor JSON builder ────────────────────────────────────────────

fn build_monitor_json(hc_with_relations: &HealthcheckWithRelations) -> Value {
    let url = hc_with_relations.url();
    let hc = &hc_with_relations.healthcheck;

    let encoding = match hc.request_body_encoding.as_str() {
        "x-www-form-urlencoded" => "form",
        "XML" => "xml",
        _ => "json",
    };

    let mut monitor = json!({
        "type": "http",
        "name": hc.name,
        "interval": hc.interval,
        "active": hc.is_enabled,
        "maxretries": hc.retry,
        "retryInterval": hc.retry_interval,
        "url": url,
        "timeout": hc.timeout_seconds,
        "method": hc.method,
        "httpBodyEncoding": encoding,
        "accepted_statuscodes": ["200-299"],
    });

    let obj = monitor.as_object_mut().unwrap();

    if let Some(ref notes) = hc.notes {
        obj.insert("description".into(), json!(notes));
    }
    if let Some(ref body) = hc.request_body {
        obj.insert("body".into(), json!(body));
    }
    if let Some(ref headers) = hc.headers {
        obj.insert("headers".into(), json!(headers));
    }

    // Authentication
    match (&hc.http_auth_user, &hc.http_auth_pass) {
        (Some(user), Some(pass)) => {
            obj.insert("authMethod".into(), json!("basic"));
            obj.insert("basic_auth_user".into(), json!(user));
            obj.insert("basic_auth_pass".into(), json!(pass));
        }
        _ => {
            obj.insert("authMethod".into(), json!("null"));
        }
    }

    monitor
}

// ── Public sync function ────────────────────────────────────────────

pub async fn sync_healthchecks_to_kuma(state: AppState) -> Result<()> {
    info!("Connecting to Kuma at {}", state.config.kuma_url);
    let client = KumaClient::connect(
        &state.config.kuma_url,
        &state.config.kuma_username,
        &state.config.kuma_password,
    )
    .await?;

    let healthchecks = service::healthcheck::get_all_with_relations(&state.pool).await?;

    for hc in healthchecks {
        let name = hc.healthcheck.name.clone();
        let kuma_id = hc.healthcheck.kuma_id;
        let mut monitor = build_monitor_json(&hc);

        if let Some(kuma_id) = kuma_id {
            monitor
                .as_object_mut()
                .unwrap()
                .insert("id".into(), json!(kuma_id));
            info!("Editing monitor '{name}' (kuma_id: {kuma_id})");
            client.edit_monitor(monitor).await?;
        } else {
            info!("Adding monitor '{name}'");
            info!("JUST KIDDING");
            // let new_id = client.add_monitor(monitor).await?;
            // info!("Created monitor '{name}' with kuma_id: {new_id}");
        }
    }

    client.disconnect().await?;
    info!("Kuma sync complete");
    Ok(())
}
