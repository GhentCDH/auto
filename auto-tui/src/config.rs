use color_eyre::eyre::{Result, eyre};
use serde::Deserialize;

/// Connection settings for the auto backend.
///
/// Resolution order (later overrides earlier):
/// 1. `~/.config/auto-tui/config.toml`
/// 2. Environment variables `AUTO_URL`, `AUTO_USERNAME`, `AUTO_PASSWORD`
/// 3. Interactive password prompt when a username is set without a password
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Config {
    pub fn load() -> Result<ResolvedConfig> {
        let mut config = Self::from_file()?.unwrap_or_default();
        config.apply_env();
        config.resolve()
    }

    fn from_file() -> Result<Option<Self>> {
        let Some(dir) = dirs::config_dir() else {
            return Ok(None);
        };
        let path = dir.join("auto-tui").join("config.toml");
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        let config = toml::from_str(&content)
            .map_err(|e| eyre!("invalid config file {}: {e}", path.display()))?;
        Ok(Some(config))
    }

    fn apply_env(&mut self) {
        if let Ok(url) = std::env::var("AUTO_URL") {
            self.url = Some(url);
        }
        if let Ok(username) = std::env::var("AUTO_USERNAME") {
            self.username = Some(username);
        }
        if let Ok(password) = std::env::var("AUTO_PASSWORD") {
            self.password = Some(password);
        }
    }

    fn resolve(self) -> Result<ResolvedConfig> {
        let url = self.url.ok_or_else(|| {
            eyre!(
                "no server url configured: set `url` in ~/.config/auto-tui/config.toml or AUTO_URL"
            )
        })?;
        let credentials = match (self.username, self.password) {
            (Some(username), Some(password)) => Some((username, password)),
            (Some(username), None) => {
                let password = rpassword::prompt_password(format!("password for {username}: "))?;
                Some((username, password))
            }
            (None, _) => None,
        };
        Ok(ResolvedConfig {
            url: url.trim_end_matches('/').to_string(),
            credentials,
        })
    }
}

/// Fully resolved configuration, ready to build an API client from.
#[derive(Debug)]
pub struct ResolvedConfig {
    pub url: String,
    pub credentials: Option<(String, String)>,
}
