use std::collections::HashMap;

use crate::api::models::{HeartbeatEntry, UptimeEvent};

/// Heartbeats kept per monitor — matches the backend window (~1 hour).
const WINDOW: usize = 120;

/// Live uptime state fed by the SSE stream, keyed by Kuma monitor id.
#[derive(Debug, Default)]
pub struct UptimeState {
    pub monitors: HashMap<i32, Vec<HeartbeatEntry>>,
    /// False until the first snapshot arrives.
    pub connected: bool,
}

impl UptimeState {
    pub fn apply(&mut self, event: UptimeEvent) {
        match event {
            // Snapshot replaces everything (sent on connect and reconnect).
            UptimeEvent::Snapshot { monitors } => {
                self.monitors = monitors
                    .into_iter()
                    .map(|(id, m)| (id, m.heartbeats))
                    .collect();
                self.connected = true;
            }
            UptimeEvent::Update { kuma_id, entry } => {
                let beats = self.monitors.entry(kuma_id).or_default();
                beats.push(entry);
                if beats.len() > WINDOW {
                    let excess = beats.len() - WINDOW;
                    beats.drain(..excess);
                }
            }
        }
    }

    pub fn heartbeats(&self, kuma_id: i32) -> &[HeartbeatEntry] {
        self.monitors
            .get(&kuma_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}
