mod commands;
mod terminal;
mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();

    utils::init_terminal()?;
    let mut terminal = terminal::PleaseTerminal::new();
    terminal.run()?;

    Ok(())
}
