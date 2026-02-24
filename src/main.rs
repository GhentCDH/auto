use tracing::info;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[tokio::main]
async fn main() -> auto::Result<()> {
    info!("Loading dot files");

    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let state = auto::AppState::new().await?;

    info!("Starting Kuma uptime poller");
    auto::kuma::spawn_kuma_poller(
        state.config.clone(),
        state.uptime_state.clone(),
        state.uptime_tx.clone(),
        state.kuma_refresh_tx.subscribe(),
    );

    info!("Starting server");

    let listener = tokio::net::TcpListener::bind(&state.config.host).await?;
    axum::serve(listener, auto::router(state)).await?;

    Ok(())
}
