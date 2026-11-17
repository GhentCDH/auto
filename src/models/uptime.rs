use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// How many heartbeats to keep in memory.
pub const HEARTBEAT_WINDOW_SIZE: usize = 120;

/// A single heartbeat record from Kuma.
///
/// Status codes: 1 = up, 0 = down, 2 = pending, 3 = maintenance.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HeartbeatEntry {
    pub status: i32,
    pub time: String,
    pub ping: Option<i32>,
    pub msg: Option<String>,
}

/// In-memory uptime state for one Kuma monitor, capped to the last hour.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MonitorUptime {
    pub kuma_id: i32,
    pub heartbeats: Vec<HeartbeatEntry>,
}

/// Events sent over SSE to Vue clients.
///
/// `Snapshot` is sent once when a client connects (full current state).
/// `Update` is sent whenever a new heartbeat arrives for one monitor.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UptimeEvent {
    Snapshot {
        monitors: HashMap<i32, MonitorUptime>,
    },
    Update {
        kuma_id: i32,
        entry: HeartbeatEntry,
    },
}

/// Current state of a healthcheck, derived from its latest Kuma heartbeat.
///
/// Variants are declared worst first; sorting by state puts problems on top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CheckState {
    Down,
    Pending,
    /// Enabled, but no heartbeat known (not synced to Kuma, or Kuma unreachable).
    Unknown,
    Maintenance,
    Up,
    Disabled,
}

impl CheckState {
    /// Map a Kuma heartbeat status code (1 up, 0 down, 2 pending, 3 maintenance).
    pub fn from_kuma(status: i32) -> Self {
        match status {
            1 => CheckState::Up,
            0 => CheckState::Down,
            2 => CheckState::Pending,
            3 => CheckState::Maintenance,
            _ => CheckState::Unknown,
        }
    }
}

/// One row of `GET /api/healthchecks/status`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HealthcheckStatus {
    pub id: String,
    pub name: String,
    pub state: CheckState,
    /// Checked URL, e.g. `https://example.org/health`.
    pub url: String,
    pub application_id: Option<String>,
    pub application_name: Option<String>,
    pub service_id: Option<String>,
    pub service_name: Option<String>,
    pub kuma_id: Option<i32>,
    /// Time of the first heartbeat in the current state. The server keeps
    /// about one hour of heartbeats, so an older start shows as that limit.
    pub since: Option<String>,
    /// Time of the latest heartbeat.
    pub last_check: Option<String>,
    /// Message of the latest heartbeat (the error, when down).
    pub message: Option<String>,
    pub ping_ms: Option<i32>,
    /// Percentage of up heartbeats in the kept window (about one hour).
    pub uptime_percent: Option<f64>,
}
