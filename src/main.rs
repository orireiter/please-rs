mod commands;
mod terminal;
mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    utils::init_terminal()?;
    terminal::PleaseTerminal::new().start()
}
