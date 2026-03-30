mod commands;
mod config;
mod history;
mod terminal;
mod utils;

use anyhow::Result;

use crate::config::PleaseConfig;

fn main() -> Result<()> {
    env_logger::init();

    utils::init_terminal()?;

    let config = PleaseConfig::get_from_filesystem();

    let mut terminal = terminal::PleaseTerminal::from_config(config)?;
    terminal.run()?;

    Ok(())
}
