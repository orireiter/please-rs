use std::io::Write;

use anyhow::Result;
use crossterm::{QueueableCommand, cursor, terminal};

pub fn init_terminal() -> Result<()> {
    terminal::enable_raw_mode()?;

    std::io::stdout()
        .queue(terminal::Clear(terminal::ClearType::All))?
        .queue(cursor::MoveTo(0, 0))?
        .queue(cursor::EnableBlinking)?
        .flush()?;

    Ok(())
}
