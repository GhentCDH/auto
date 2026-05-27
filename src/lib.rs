use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use hickory_resolver::TokioResolver;
use sqlx::SqlitePool;
use tokio::sync::{RwLock, broadcast, watch};
use tracing::info;

mod api;
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
        let config = Config::from_env()?;

        info!("Connecting to database");

        let pool = SqlitePool::connect(&config.database_url).await?;

        info!("Running migrations");

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        let (uptime_tx, _) = broadcast::channel::<UptimeEvent>(64);
        let uptime_state: UptimeState = Arc::new(RwLock::new(HashMap::new()));
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
}
