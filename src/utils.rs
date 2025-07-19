use std::io::Write;

use anyhow::Result;
use crossterm::{QueueableCommand, cursor, terminal};

pub const NEWLINE: &str = "\n";
pub const HOME_DIR: &str = "~/";

pub struct ClearOptions {
    clear_type: terminal::ClearType,
}

impl ClearOptions {
    pub fn new(clear_type: terminal::ClearType) -> Self {
        Self { clear_type }
    }
}

impl Default for ClearOptions {
    fn default() -> Self {
        Self {
            clear_type: terminal::ClearType::All,
        }
    }
}

pub fn clear_terminal(clear_options: Option<ClearOptions>) -> std::io::Result<()> {
    let clear_options = clear_options.unwrap_or_default();

    let mut stdout = std::io::stdout();
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;

    if clear_options.clear_type != terminal::ClearType::All {
        stdout.queue(terminal::Clear(clear_options.clear_type))?;
    }

    stdout
        .queue(cursor::MoveTo(0, 0))?
        .queue(cursor::EnableBlinking)?
        .flush()
}

pub fn init_terminal() -> Result<()> {
    terminal::enable_raw_mode()?;
    clear_terminal(None)?;

    Ok(())
}
