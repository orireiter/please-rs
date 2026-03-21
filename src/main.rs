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
    let history_object = history::History::from_config(config.history)?;
    let command_object = commands::LiveCommand::from_config(config.command);

    let mut terminal = terminal::PleaseTerminal::new(history_object, command_object);
    terminal.run()?;

    Ok(())
}
