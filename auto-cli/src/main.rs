mod api;
mod app;
mod cli;
mod config;
mod event;
mod login;
mod mcp;
mod openapi;
mod ops;
mod output;
mod session_store;
mod tui;
mod ui;
mod update_check;
mod view;

use clap::Parser;

#[tokio::main]
async fn main() {
    let cli::Cli { global, command } = cli::Cli::parse();
    let result = match command {
        None => {
            let _ = color_eyre::install();
            tui::run(global.url.as_deref()).await
        }
        Some(command) => cli::run(command, &global).await,
    };
    if let Err(error) = result {
        // The reader went away (e.g. `auto list apps | head`): not an error.
        if error
            .downcast_ref::<std::io::Error>()
            .is_some_and(|e| e.kind() == std::io::ErrorKind::BrokenPipe)
        {
            return;
        }
        // `{:#}` prints the error chain on one line, without the report
        // decoration, which reads better in a terminal and in agent logs.
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
