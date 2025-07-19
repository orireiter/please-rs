mod commands;
mod history;
mod terminal;
mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();

    utils::init_terminal()?;

    let history_config = history::HistoryConfig::default();
    let _history_object = history::History::from_config(history_config)?;

    let mut terminal = terminal::PleaseTerminal::new();
    terminal.run()?;

    Ok(())
}
