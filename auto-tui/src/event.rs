//! Merged event stream: terminal input, ticks, API results and SSE updates
//! all arrive on one channel so the draw loop never awaits I/O directly.

use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use futures::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::api::EntityKind;
use crate::api::models::*;

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Resize,
    Tick,
    Data(DataMsg),
    Uptime(UptimeEvent),
    Error(String),
}

/// Results of background API calls, tagged so the app can route them to the
/// right screen state.
#[derive(Debug)]
pub enum DataMsg {
    Dashboard(DashboardStats),
    List {
        entity: EntityKind,
        resp: PaginatedResponse<Value>,
    },
    Detail {
        entity: EntityKind,
        value: Value,
    },
    Search(SearchResults),
    Dns(DnsLookup),
    ExecResult(HealthcheckExecuteResult),
    ActionDone {
        label: String,
    },
}

/// Spawn the input-reader and tick producers onto `tx`.
pub fn spawn_producers(tx: mpsc::UnboundedSender<Event>) {
    let input_tx = tx.clone();
    tokio::spawn(async move {
        let mut events = crossterm::event::EventStream::new();
        while let Some(Ok(event)) = events.next().await {
            let mapped = match event {
                CrosstermEvent::Key(key) => Event::Key(key),
                CrosstermEvent::Resize(..) => Event::Resize,
                _ => continue,
            };
            if input_tx.send(mapped).is_err() {
                break;
            }
        }
    });

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));
        loop {
            interval.tick().await;
            if tx.send(Event::Tick).is_err() {
                break;
            }
        }
    });
}
