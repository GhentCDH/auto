mod api;
mod app;
mod config;
mod event;
mod ui;

use color_eyre::eyre::Result;
use tokio::sync::mpsc;

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
    let terminal = ratatui::init();
    let result = run(terminal, client).await;
    ratatui::restore();
    result
}

async fn run(mut terminal: ratatui::DefaultTerminal, client: api::ApiClient) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();
    event::spawn_producers(tx.clone());

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
