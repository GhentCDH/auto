//! Tell the user when the server runs a newer release than this CLI.
//!
//! The CLI and the server share one version (one release tag builds both),
//! so a newer server means a newer CLI is available. The check runs at most
//! once a day, with a short timeout, and never fails a command.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::Method;
use serde_json::Value;

use crate::api::ApiClient;
use crate::config::APP_DIR;

const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);
const TIMEOUT: Duration = Duration::from_secs(2);
pub const INSTALL_COMMAND: &str =
    "curl -fsSL https://raw.githubusercontent.com/GhentCDH/auto/main/scripts/install.sh | sh";

pub async fn notify_if_outdated(client: &ApiClient) {
    let Some(stamp) = dirs::cache_dir().map(|dir| dir.join(APP_DIR).join("last-version-check"))
    else {
        return;
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let last = std::fs::read_to_string(&stamp)
        .ok()
        .and_then(|text| text.trim().parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or_default();
    if now.saturating_sub(last) < CHECK_EVERY {
        return;
    }

    let request = client.call(Method::GET, "/version", &[], None);
    let Ok(Ok(response)) = tokio::time::timeout(TIMEOUT, request).await else {
        return;
    };
    if let Some(dir) = stamp.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&stamp, now.as_secs().to_string());

    let server = response
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("");
    let cli = env!("CARGO_PKG_VERSION");
    if let (Some(server_parts), Some(cli_parts)) = (parse(server), parse(cli))
        && server_parts > cli_parts
    {
        eprintln!(
            "note: the server runs auto {server}, this CLI is {cli}. Update with:\n  {INSTALL_COMMAND}"
        );
    }
}

fn parse(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version
        .trim_start_matches('v')
        .split(['.', '-', '+'])
        .map(|part| part.parse::<u64>().ok());
    Some((parts.next()??, parts.next()??, parts.next()??))
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn compares_numerically() {
        assert!(parse("1.15.0") > parse("1.14.9"));
        assert!(parse("1.10.0") > parse("1.9.3"));
        assert!(parse("v2.0.0") > parse("1.99.0"));
        assert_eq!(parse("1.14.0-rc1"), Some((1, 14, 0)));
        assert_eq!(parse("1.14"), None);
        assert_eq!(parse("dev"), None);
    }
}
