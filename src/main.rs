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

    info!("Starting server");

    let listener = tokio::net::TcpListener::bind(&state.config.domain).await?;
    axum::serve(listener, auto::router(state)).await?;

    Ok(())
}
