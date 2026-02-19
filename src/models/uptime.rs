use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// How many seconds of heartbeat history to keep in memory (1 hour).
pub const HEARTBEAT_WINDOW_SECS: i64 = 3600;

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
