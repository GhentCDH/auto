use tracing::info;

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub domain: String,
    pub database_url: String,
    pub kuma_url: kuma_client::Url,
    pub kuma_username: String,
    pub kuma_password: String,
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

        Ok(Self {
            domain: var("DOMAIN"),
            database_url: var("DATABASE_URL"),
            kuma_url: kuma_client::Url::parse(&var("KUMA_URL"))
                .expect("KUMA_URL should be a valid kuma URL"),
            kuma_username: var("KUMA_USERNAME"),
            kuma_password: var("KUMA_PASSWORD"),
        })
    }
}
