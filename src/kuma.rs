/*!
 * Custom Uptime Kuma client using Socket.IO.
 *
 * Replaces the `kuma-client` crate which panicked on float deserialization.
 * Only implements the subset needed: connect, login, edit/add monitors.
 *
 * The database of Auto is the single source of truth — this module
 * pushes healthcheck state to Kuma.
 *
 * Also contains the persistent Kuma poller that reads heartbeat data
 * from Kuma and broadcasts it via SSE to connected clients.
 */

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use futures::FutureExt as _;
use rust_socketio::{
    Payload,
    asynchronous::{Client as SocketClient, ClientBuilder},
};
use serde_json::{Value, json};
use tokio::sync::{RwLock, broadcast, mpsc, watch};
use tracing::{debug, error, info, trace, warn};

use crate::{
    AppState, Error, Result,
    models::{
        HEARTBEAT_WINDOW_SECS, HealthcheckWithRelations, HeartbeatEntry, MonitorUptime,
        UpdateHealthcheck, UptimeEvent,
    },
    service,
};

// ── Public type aliases ────────────────────────────────────────────

pub type UptimeState = Arc<RwLock<HashMap<i32, MonitorUptime>>>;
pub type UptimeTx = broadcast::Sender<UptimeEvent>;

// ── KumaClient (unchanged) ────────────────────────────────────────

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
                    trace!("Kuma event: {event}");
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
        "accepted_statuscodes": [hc.expected_status.to_string()],
        "conditions": "[]"
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
    debug!("Connecting to Kuma at {}", state.config.kuma_url);
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
            debug!("Editing monitor '{name}' (kuma_id: {kuma_id})");
            client.edit_monitor(monitor).await?;
        } else {
            debug!("Adding monitor '{name}'");
            let new_id = client.add_monitor(monitor).await?;
            debug!("Created monitor '{name}' with kuma_id: {new_id}");
            service::healthcheck::update(
                &state.pool,
                &hc.healthcheck.id,
                UpdateHealthcheck {
                    kuma_id: Some(new_id),
                    ..Default::default()
                },
            )
            .await?;
            debug!("Updated Auto healthcheck's kuma_id");
        }
    }

    client.disconnect().await?;
    debug!("Kuma sync complete");
    Ok(())
}

// ── Persistent Kuma Poller ──────────────────────────────────────────

/// Spawns the persistent Kuma poller on a dedicated OS thread.
///
/// The poller maintains a Socket.IO connection to Kuma, listens for
/// heartbeat events, and broadcasts them to SSE clients. It reconnects
/// automatically on disconnect with exponential backoff, and reconnects
/// immediately when notified of a sync (to pick up new kuma_ids).
pub fn spawn_kuma_poller(
    config: crate::Config,
    uptime_state: UptimeState,
    uptime_tx: UptimeTx,
    refresh_rx: watch::Receiver<()>,
) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Kuma poller runtime");
        rt.block_on(run_poller_loop(config, uptime_state, uptime_tx, refresh_rx));
    });
}

struct ReconnectPolicy {
    base_delay_secs: u64,
    max_delay_secs: u64,
    current_delay_secs: u64,
}

impl ReconnectPolicy {
    fn new() -> Self {
        Self {
            base_delay_secs: 5,
            max_delay_secs: 300,
            current_delay_secs: 5,
        }
    }

    fn next_delay(&mut self) -> Duration {
        let d = Duration::from_secs(self.current_delay_secs);
        self.current_delay_secs = (self.current_delay_secs * 2).min(self.max_delay_secs);
        d
    }

    fn reset(&mut self) {
        self.current_delay_secs = self.base_delay_secs;
    }
}

async fn run_poller_loop(
    config: crate::Config,
    uptime_state: UptimeState,
    uptime_tx: UptimeTx,
    mut refresh_rx: watch::Receiver<()>,
) {
    let mut policy = ReconnectPolicy::new();

    loop {
        tokio::select! {
            result = connect_and_poll(&config, uptime_state.clone(), uptime_tx.clone()) => {
                match result {
                    Ok(()) => policy.reset(),
                    Err(e) => {
                        error!("Kuma poller error: {e}");
                        let delay = policy.next_delay();
                        warn!("Reconnecting to Kuma in {delay:?}");
                        tokio::time::sleep(delay).await;
                    }
                }
            }
            _ = refresh_rx.changed() => {
                info!("Kuma refresh triggered: reconnecting poller");
                // Brief pause to let the sync finish writing new kuma_ids to DB
                tokio::time::sleep(Duration::from_secs(2)).await;
                policy.reset();
            }
        }
    }
}

async fn connect_and_poll(
    config: &crate::Config,
    uptime_state: UptimeState,
    uptime_tx: UptimeTx,
) -> Result<()> {
    info!("Kuma poller: connecting to {}", config.kuma_url);

    // We need to register event handlers BEFORE calling connect(), because
    // Kuma starts pushing events immediately after the connection is established.
    let ready = Arc::new(tokio::sync::Notify::new());
    let ready_signal = ready.clone();

    // Clone state/tx for each handler closure (they need 'static + Send)
    let state_for_hb_list = uptime_state.clone();
    let tx_for_hb_list = uptime_tx.clone();
    let state_for_hb = uptime_state.clone();
    let tx_for_hb = uptime_tx.clone();

    let socket = ClientBuilder::new(config.kuma_url.as_str())
        .on_any(move |event, _payload, _client| {
            let ready = ready_signal.clone();
            async move {
                trace!("Kuma poller event: {event}");
                ready.notify_one();
            }
            .boxed()
        })
        .on("heartbeatList", move |payload: Payload, _client| {
            let state = state_for_hb_list.clone();
            let tx = tx_for_hb_list.clone();
            async move {
                if let Payload::Text(values) = payload {
                    handle_heartbeat_list(values, state, tx).await;
                }
            }
            .boxed()
        })
        .on("heartbeat", move |payload: Payload, _client| {
            let state = state_for_hb.clone();
            let tx = tx_for_hb.clone();
            async move {
                if let Payload::Text(values) = payload {
                    handle_heartbeat(values, state, tx).await;
                }
            }
            .boxed()
        })
        .connect()
        .await
        .map_err(|e| Error::KumaError(format!("Poller connection failed: {e}")))?;

    // Wait for readiness (Kuma sends "info" event)
    tokio::time::timeout(Duration::from_secs(15), ready.notified())
        .await
        .map_err(|_| Error::KumaError("Poller timed out waiting for Kuma ready".into()))?;

    // Authenticate — inline login (can't reuse KumaClient since we built the socket differently)
    let (login_tx, mut login_rx) = mpsc::channel::<Value>(1);
    let username = config.kuma_username.clone();
    let password = config.kuma_password.clone();

    socket
        .emit_with_ack(
            "login",
            Payload::Text(vec![json!({
                "username": username,
                "password": password,
                "token": ""
            })]),
            Duration::from_secs(30),
            move |msg: Payload, _| {
                let tx = login_tx.clone();
                async move {
                    if let Payload::Text(vs) = msg {
                        let json = Value::Array(vs);
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
        .map_err(|e| Error::KumaError(format!("Poller login emit failed: {e}")))?;

    let login_resp = login_rx
        .recv()
        .await
        .ok_or_else(|| Error::KumaError("No poller login ack".into()))?;

    if login_resp.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        let msg = login_resp
            .get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        return Err(Error::KumaError(format!("Poller login failed: {msg}")));
    }

    info!("Kuma poller: authenticated, listening for heartbeats");

    // Keep the socket alive. The on("heartbeatList") and on("heartbeat")
    // handlers fire asynchronously. We block here indefinitely; the caller's
    // loop handles reconnect on drop/error.
    futures::future::pending::<()>().await;

    // Unreachable, but satisfies the return type
    let _ = socket.disconnect().await;
    Ok(())
}

// ── Heartbeat handlers ──────────────────────────────────────────────

/// Processes `heartbeatList` — bulk history sent by Kuma on connect after login.
///
/// Payload shape: `[{ "1": [{status, time, ping, msg}, ...], "2": [...] }]`
/// Keys are monitor IDs as strings (JS object keys are always strings).
async fn handle_heartbeat_list(values: Vec<Value>, state: UptimeState, tx: UptimeTx) {
    let data = match values.into_iter().next() {
        Some(v) => v,
        None => return,
    };
    let obj = match data.as_object() {
        Some(o) => o,
        None => return,
    };

    let cutoff = Utc::now() - chrono::Duration::seconds(HEARTBEAT_WINDOW_SECS);

    let mut write = state.write().await;
    for (monitor_id_str, beats_val) in obj {
        let kuma_id: i32 = match monitor_id_str.parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        let beats_arr = match beats_val.as_array() {
            Some(a) => a,
            None => continue,
        };

        let mut heartbeats: Vec<HeartbeatEntry> = beats_arr
            .iter()
            .filter_map(|v| parse_heartbeat_entry(v))
            .filter(|h| {
                DateTime::parse_from_rfc3339(&h.time)
                    .map(|t| t.with_timezone(&Utc) > cutoff)
                    .unwrap_or(true) // keep if time is unparseable
            })
            .collect();

        heartbeats.sort_by(|a, b| a.time.cmp(&b.time));

        write.insert(
            kuma_id,
            MonitorUptime {
                kuma_id,
                heartbeats,
            },
        );
    }
    drop(write);

    // Broadcast snapshot to all connected SSE clients
    let snapshot = {
        let read = state.read().await;
        read.clone()
    };
    let _ = tx.send(UptimeEvent::Snapshot { monitors: snapshot });

    info!(
        "Kuma poller: received heartbeat list for {} monitors",
        obj.len()
    );
}

/// Processes `heartbeat` — a single real-time heartbeat from Kuma.
///
/// Payload shape: `[{ monitorID: 42, status: 1, time: "...", ping: 120, msg: "" }]`
/// Note: Kuma uses `monitorID` (capital D) here, not `monitorId`.
async fn handle_heartbeat(values: Vec<Value>, state: UptimeState, tx: UptimeTx) {
    let data = match values.into_iter().next() {
        Some(v) => v,
        None => return,
    };

    let kuma_id = match data.get("monitorID").and_then(|v| v.as_i64()) {
        Some(id) => id as i32,
        None => return,
    };
    let entry = match parse_heartbeat_entry(&data) {
        Some(e) => e,
        None => return,
    };

    let cutoff = Utc::now() - chrono::Duration::seconds(HEARTBEAT_WINDOW_SECS);

    {
        let mut write = state.write().await;
        let monitor = write.entry(kuma_id).or_insert(MonitorUptime {
            kuma_id,
            heartbeats: Vec::new(),
        });
        monitor.heartbeats.push(entry.clone());

        // Prune entries older than the window
        monitor.heartbeats.retain(|h| {
            DateTime::parse_from_rfc3339(&h.time)
                .map(|t| t.with_timezone(&Utc) > cutoff)
                .unwrap_or(true)
        });
    }

    let _ = tx.send(UptimeEvent::Update { kuma_id, entry });
}

/// Parses a JSON value into a HeartbeatEntry.
///
/// Handles Kuma quirks: `ping` can be a float (e.g. 42.7), integer, or null.
fn parse_heartbeat_entry(v: &Value) -> Option<HeartbeatEntry> {
    let status = v.get("status")?.as_i64()? as i32;
    let time = format!("{}Z", v.get("time")?.as_str()?);
    let ping = v.get("ping").and_then(|p| {
        if p.is_null() {
            None
        } else {
            p.as_f64().map(|f| f as i32)
        }
    });
    let msg = v.get("msg").and_then(|m| {
        if m.is_null() {
            None
        } else {
            m.as_str().map(|s| s.to_string())
        }
    });
    Some(HeartbeatEntry {
        status,
        time,
        ping,
        msg,
    })
}
