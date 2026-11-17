use std::path::PathBuf;

use color_eyre::eyre::{Result, eyre};
use serde::Deserialize;

/// Directory name under the OS config and data dirs.
pub const APP_DIR: &str = "auto";

/// Connection settings for the auto backend.
///
/// Resolution order (later overrides earlier):
/// 1. `~/.config/auto/config.toml`
/// 2. Environment variables `AUTO_URL`, `AUTO_USERNAME`, `AUTO_PASSWORD`,
///    `AUTO_TOKEN`
///
/// Authentication is optional: a server with auth disabled needs none. When
/// auth is enabled the client tries, in order: a configured `token` (personal
/// API token, also works for OIDC/SSO accounts), a session stored from a
/// previous browser login, then the login screen (browser loopback first,
/// password second, prefilled from `username`/`password`).
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    /// Local checkouts of central deployment repos (nomad jobs, proxy
    /// config, ...). `auto here` lists them so an audit can search them.
    #[serde(default)]
    pub deploy_repos: Vec<String>,
}

impl Config {
    /// Load the file and environment; `url` (from `--url`) wins over both.
    pub fn load(url: Option<&str>) -> Result<ResolvedConfig> {
        let mut config = Self::from_file()?.unwrap_or_default();
        config.apply_env();
        if let Some(url) = url {
            config.url = Some(url.to_string());
        }
        config.resolve()
    }

    /// `deploy_repos` from the config file alone (no server url needed).
    pub fn deploy_repo_paths() -> Result<Vec<PathBuf>> {
        let config = Self::from_file()?.unwrap_or_default();
        Ok(config.deploy_repos.iter().map(|p| expand_home(p)).collect())
    }

    fn from_file() -> Result<Option<Self>> {
        let Some(path) = config_path().filter(|path| path.exists()) else {
            return Ok(None);
        };
        let content = std::fs::read_to_string(&path)?;
        let config = toml::from_str(&content)
            .map_err(|e| eyre!("invalid config file {}: {e}", path.display()))?;
        Ok(Some(config))
    }

    fn apply_env(&mut self) {
        let var = |name| std::env::var(name).ok();
        self.url = var("AUTO_URL").or(self.url.take());
        self.username = var("AUTO_USERNAME").or(self.username.take());
        self.password = var("AUTO_PASSWORD").or(self.password.take());
        self.token = var("AUTO_TOKEN").or(self.token.take());
    }

    fn resolve(self) -> Result<ResolvedConfig> {
        let url = self.url.ok_or_else(|| {
            eyre!(
                "no server url configured: run `auto login --url <url>`, set `url` in \
                 ~/.config/auto/config.toml, or set AUTO_URL"
            )
        })?;
        Ok(ResolvedConfig {
            url: normalize_url(&url),
            token: self.token,
            username: self.username,
            password: self.password,
            deploy_repos: self.deploy_repos.iter().map(|p| expand_home(p)).collect(),
        })
    }
}

/// Expand a leading `~/` to the home directory.
fn expand_home(path: &str) -> PathBuf {
    match (path.strip_prefix("~/"), dirs::home_dir()) {
        (Some(rest), Some(home)) => home.join(rest),
        _ => PathBuf::from(path),
    }
}

/// A bare host like "auto.example.org" is fine: assume https.
pub fn normalize_url(url: &str) -> String {
    let url = url.trim_end_matches('/');
    if url.contains("://") {
        url.to_string()
    } else {
        format!("https://{url}")
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join(APP_DIR).join("config.toml"))
}

/// Set keys in `~/.config/auto/config.toml`, keeping all other keys. The
/// file gets owner-only permissions, since it can hold a token.
pub fn save_values(values: &[(&str, &str)]) -> Result<PathBuf> {
    let target = config_path().ok_or_else(|| eyre!("no config directory on this system"))?;
    let mut table: toml::Table = match target.exists() {
        true => toml::from_str(&std::fs::read_to_string(&target)?)
            .map_err(|e| eyre!("invalid config file {}: {e}", target.display()))?,
        false => toml::Table::new(),
    };
    values.iter().for_each(|(key, value)| {
        table.insert(key.to_string(), toml::Value::String(value.to_string()));
    });
    crate::session_store::write_private(&target, toml::to_string(&table)?.as_bytes())?;
    Ok(target)
}

/// Fully resolved configuration, ready to build an API client from.
#[derive(Debug)]
pub struct ResolvedConfig {
    pub url: String,
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub deploy_repos: Vec<PathBuf>,
}
