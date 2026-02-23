use std::collections::HashMap;
use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::{RwLock, broadcast, watch};
use tracing::info;

mod api;
mod config;
mod error;
pub mod kuma;
pub mod models;
mod openapi;
mod overview;
mod routes;
mod service;

pub use config::Config;
pub use error::Error;
pub use openapi::ApiDoc;
pub use routes::router;

pub type Result<T> = std::result::Result<T, Error>;

use kuma::{UptimeState, UptimeTx};
use models::UptimeEvent;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Config,
    /// Shared in-memory uptime state (last ~1 hour per monitor).
    pub uptime_state: UptimeState,
    /// Broadcast channel for SSE fan-out.
    pub uptime_tx: UptimeTx,
    /// Notifies the Kuma poller to reconnect after a sync.
    pub kuma_refresh_tx: watch::Sender<()>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let config = Config::from_env()?;

        info!("Connecting to database");

        let pool = SqlitePool::connect(&config.database_url).await?;

        info!("Running migrations");

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        let (uptime_tx, _) = broadcast::channel::<UptimeEvent>(64);
        let uptime_state: UptimeState = Arc::new(RwLock::new(HashMap::new()));
        let (kuma_refresh_tx, _) = watch::channel(());

        let state = Self {
            pool,
            config,
            uptime_state,
            uptime_tx,
            kuma_refresh_tx,
        };

        Ok(state)
    }
}
