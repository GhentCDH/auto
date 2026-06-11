use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;
use tracing::info;
use url::Url;

use crate::error::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub database_url: String,
    pub kuma_url: Url,
    pub kuma_username: String,
    pub kuma_password: String,
    #[serde(default)]
    pub kuma_notification_name: Option<String>,
    #[serde(default)]
    pub outline_url: Option<Url>,
    #[serde(default)]
    pub outline_api_key: Option<String>,
    /// How many days before an infra's domain-resolved IPs are considered stale
    /// and re-resolved on read. Optional `INFRA_IP_REFRESH_DAYS` (default 10).
    #[serde(default = "default_infra_ip_refresh_days")]
    pub infra_ip_refresh_days: u64,
}

fn default_infra_ip_refresh_days() -> u64 {
    10
}

impl Config {
    /// Load configuration by layering an optional `auto.toml` file under the
    /// process environment. Environment variables override TOML on conflict.
    ///
    /// `.env` (falling back to `dev.env`) is loaded into the process environment
    /// first, so file-based secrets keep working.
    ///
    /// # Errors
    /// If a required setting is missing or a value has the wrong format.
    pub fn load() -> Result<Self, Error> {
        info!("loading configuration (auto.toml < env)");
        if dotenvy::dotenv().is_err() {
            info!(".env not found, defaulting to dev.env");
            if dotenvy::from_path("dev.env").is_err() {
                info!("dev.env not found");
            }
        }

        Figment::new()
            .merge(Toml::file("auto.toml"))
            .merge(Env::raw())
            .extract()
            .map_err(Error::from)
    }
}
