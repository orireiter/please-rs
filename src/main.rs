mod commands;
mod terminal;
mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();

    utils::init_terminal()?;
    terminal::PleaseTerminal::new().start()
}
