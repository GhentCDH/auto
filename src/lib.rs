use sqlx::SqlitePool;
use tracing::info;

mod api;
mod config;
mod error;
mod kuma;
pub mod models;
mod routes;
mod service;

pub use config::Config;
pub use error::Error;
pub use routes::router;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Config,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let config = Config::from_env()?;

        info!("Connecting to database");

        let pool = SqlitePool::connect(&config.database_url).await?;

        info!("Running migrations");

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        let state = Self { pool, config };

        Ok(state)
    }
}
