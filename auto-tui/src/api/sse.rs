//! SSE consumer for `/healthchecks/uptime/stream`.
//!
//! Manual parsing over a byte stream: the backend's framing is plain
//! `data: <json>` lines, blank-line terminated, with `:` keep-alive comments.
//! Reconnects with capped backoff; each reconnect re-delivers a full
//! snapshot which replaces the uptime state.

use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;

use crate::api::ApiClient;
use crate::api::models::UptimeEvent;
use crate::event::Event;

pub fn spawn(client: ApiClient, tx: UnboundedSender<Event>) {
    tokio::spawn(async move {
        let mut backoff_secs = 1u64;
        loop {
            match stream_once(&client, &tx).await {
                // Channel closed: the app is shutting down.
                Err(StreamError::ChannelClosed) => return,
                // Connection dropped or failed: retry with backoff.
                Err(StreamError::Connection(_)) | Ok(()) => {}
            }
            tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
            backoff_secs = (backoff_secs * 2).min(30);
        }
    });
}

enum StreamError {
    Connection(String),
    ChannelClosed,
}

async fn stream_once(client: &ApiClient, tx: &UnboundedSender<Event>) -> Result<(), StreamError> {
    let response = client
        .uptime_stream_request()
        .send()
        .await
        .map_err(|e| StreamError::Connection(e.to_string()))?;
    if !response.status().is_success() {
        return Err(StreamError::Connection(format!(
            "uptime stream failed: {}",
            response.status()
        )));
    }

    let mut bytes = response.bytes_stream();
    let mut buffer = String::new();
    let mut data = String::new();

    while let Some(chunk) = bytes.next().await {
        let chunk = chunk.map_err(|e| StreamError::Connection(e.to_string()))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process every complete line currently in the buffer.
        while let Some(newline) = buffer.find('\n') {
            let line = buffer[..newline].trim_end_matches('\r').to_string();
            buffer.drain(..=newline);

            if let Some(payload) = line.strip_prefix("data:") {
                if !data.is_empty() {
                    data.push('\n');
                }
                data.push_str(payload.strip_prefix(' ').unwrap_or(payload));
            } else if line.is_empty() && !data.is_empty() {
                // Blank line terminates one SSE event.
                if let Ok(event) = serde_json::from_str::<UptimeEvent>(&data)
                    && tx.send(Event::Uptime(event)).is_err()
                {
                    return Err(StreamError::ChannelClosed);
                }
                data.clear();
            }
            // `:` comments (keep-alives) and other fields are ignored.
        }
    }
    Ok(())
}
