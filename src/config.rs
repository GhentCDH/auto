use tracing::info;
use url::Url;

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: String,
    pub base_url: String,
    pub database_url: String,
    pub kuma_url: Url,
    pub kuma_username: String,
    pub kuma_password: String,
    pub kuma_notification_name: Option<String>,
    pub outline_url: Option<Url>,
    pub outline_api_key: Option<String>,
    /// How many days before an infra's domain-resolved IPs are considered stale
    /// and re-resolved on read. Optional env `INFRA_IP_REFRESH_DAYS` (default 10).
    pub infra_ip_refresh_days: u64,
}

/// # Panics
/// If the environment variable does not exist
fn var(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Environment variable `{name}` should be set"))
}

impl Config {
    /// # Panics
    /// If one of the required environment variables has not been set or has the wrong format.
    pub fn from_env() -> Result<Self, Error> {
        info!("loading environment variables from .env");
        if dotenvy::dotenv().is_err() {
            info!(".env not found, defaulting to dev.env");
            if dotenvy::from_path("dev.env").is_err() {
                info!("dev.env not found");
            }
        }

        let outline_url = std::env::var("OUTLINE_URL")
            .ok()
            .and_then(|u| Url::parse(&u).ok());
        let outline_api_key = std::env::var("OUTLINE_API_KEY").ok();

        let infra_ip_refresh_days = std::env::var("INFRA_IP_REFRESH_DAYS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(10);

        Ok(Self {
            host: var("HOST"),
            port: var("PORT"),
            base_url: var("BASE_URL"),
            database_url: var("DATABASE_URL"),
            kuma_url: Url::parse(&var("KUMA_URL")).expect("KUMA_URL should be a valid URL"),
            kuma_username: var("KUMA_USERNAME"),
            kuma_password: var("KUMA_PASSWORD"),
            kuma_notification_name: std::env::var("KUMA_NOTIFICATION_NAME").ok(),
            outline_url,
            outline_api_key,
            infra_ip_refresh_days,
        })
    }
}
