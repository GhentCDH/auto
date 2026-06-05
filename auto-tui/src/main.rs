mod config;

use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let config = config::Config::load()?;
    println!("auto-tui — connecting to {}", config.url);
    Ok(())
}
