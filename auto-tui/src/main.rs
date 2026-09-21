mod api;
mod app;
mod config;
mod event;
mod login;
mod session_store;
mod ui;

use color_eyre::eyre::Result;
use tokio::sync::mpsc;

use api::AuthStatus;
use app::App;
use event::Event;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // Resolve config (may prompt for a password) before entering raw mode.
    let config = config::Config::load()?;
    let client = api::ApiClient::new(&config)?;

    // ratatui::init installs a panic hook that restores the terminal first,
    // so color-eyre reports stay readable after a crash.
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, client).await;
    ratatui::restore();
    result
}

async fn run(terminal: &mut ratatui::DefaultTerminal, client: api::ApiClient) -> Result<()> {
    // Authenticate before the main UI: open mode or an existing token/session
    // pass silently, otherwise show the login screen (loopback browser first).
    match client.try_silent_auth().await? {
        AuthStatus::Authenticated => {}
        AuthStatus::LoginRequired { reason } => {
            if !login::run(terminal, &client, reason).await? {
                return Ok(());
            }
        }
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();
    event::spawn_producers(tx.clone());
    api::sse::spawn(client.clone(), tx.clone());

    let mut app = App::new(client, tx);
    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;
        let Some(event) = rx.recv().await else {
            break;
        };
        app.handle_event(event);
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
