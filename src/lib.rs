use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use hickory_resolver::TokioResolver;
use sqlx::SqlitePool;
use tokio::sync::{RwLock, broadcast, watch};
use tracing::info;

mod api;
pub mod auth;
mod config;
mod error;
pub mod kuma;
pub mod models;
mod openapi;
pub mod outline;
pub mod overview;
mod routes;
mod service;

pub use config::Config;
pub use error::Error;
pub use openapi::ApiDoc;
pub use routes::router;

pub type Result<T> = std::result::Result<T, Error>;

use kuma::{UptimeState, UptimeTx};
use models::{DnsRecord, UptimeEvent};

use crate::kuma::UptimeStateInner;

/// Shared cache of live DNS lookups, keyed by FQDN.
/// Holds the monotonic fill instant (for TTL), the RFC3339 wall-clock resolve time
/// (for display), and the records, so [`service::dns`] can honor a short TTL.
pub type DnsCache = Arc<RwLock<HashMap<String, (Instant, String, Vec<DnsRecord>)>>>;

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
    /// System-configured DNS resolver for live domain record lookups.
    pub resolver: Arc<TokioResolver>,
    /// Short-TTL cache of live DNS lookups, keyed by FQDN.
    pub dns_cache: DnsCache,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let config = Config::load()?;

        info!("Running migrations");

        // Migrations run on a dedicated connection with foreign-key enforcement
        // disabled. Some migrations rebuild a referenced table (e.g. domain) via
        // drop/rename; with FK on, sqlx's transactional apply would fire implicit
        // DELETE cascades/RESTRICT against child tables. foreign_keys is a
        // connection-level pragma (set before any transaction), so disabling it
        // here does not affect the runtime pool below, which keeps FK on.
        {
            use sqlx::{ConnectOptions, Connection};
            use std::str::FromStr;
            let mut mig_conn = sqlx::sqlite::SqliteConnectOptions::from_str(&config.database_url)?
                .foreign_keys(false)
                .connect()
                .await?;
            sqlx::migrate!("./migrations").run(&mut mig_conn).await?;
            mig_conn.close().await?;
        }

        info!("Connecting to database");

        let pool = SqlitePool::connect(&config.database_url).await?;

        let (uptime_tx, _) = broadcast::channel::<UptimeEvent>(64);
        let uptime_state: UptimeState = Arc::new(RwLock::new(UptimeStateInner {
            uptimes: HashMap::new(),
            notification_handlers: HashMap::new(),
        }));
        let (kuma_refresh_tx, _) = watch::channel(());

        info!("Building DNS resolver from system config");

        let resolver = Arc::new({
            let mut builder = TokioResolver::builder_tokio()
                .map_err(|e| Error::InternalError(format!("DNS resolver init failed: {e}")))?;
            {
                // Bound the worst case: one attempt, moderate timeout. And query all
                // configured nameservers in parallel so a dead/slow entry in
                // resolv.conf (e.g. an unroutable `::`) loses the race instead of
                // blocking the whole lookup for timeout × attempts.
                let opts = builder.options_mut();
                opts.attempts = 1;
                opts.timeout = std::time::Duration::from_secs(3);
                opts.num_concurrent_reqs = 5;
            }
            builder
                .build()
                .map_err(|e| Error::InternalError(format!("DNS resolver init failed: {e}")))?
        });
        let dns_cache: DnsCache = Arc::new(RwLock::new(HashMap::new()));

        let state = Self {
            pool,
            config,
            uptime_state,
            uptime_tx,
            kuma_refresh_tx,
            resolver,
            dns_cache,
        };

        Ok(state)
    }

    pub async fn get_kuma_notification_handler_id(&self) -> Option<i32> {
        if let Some(n) = &self.config.kuma_notification_name {
            self.uptime_state
                .read()
                .await
                .find_notification_handler(n)
                .await
        } else {
            None
        }
    }
}
