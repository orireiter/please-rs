use std::{fmt::Display, io::Write};

use anyhow::{Context, Result};
use crossterm::{ExecutableCommand, QueueableCommand, cursor, style::StyledContent, terminal};
use itertools::intersperse;

pub const SPACE: &str = " ";
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

    ctrlc::set_handler(|| {}).context("failed setting ctrl+c handler")?;

    Ok(())
}

pub fn move_left(stdout: &mut std::io::Stdout) -> Result<()> {
    let (x, y) = cursor::position()?;

    if x == 0 {
        if y == 0 {
            return Ok(());
        }

        let terminal_size = terminal::size()?;
        stdout.execute(cursor::MoveTo(terminal_size.0, y - 1))?;
    } else {
        stdout.execute(cursor::MoveLeft(1))?;
    }

    Ok(())
}

#[derive(Default)]
pub struct StyledContentGroup {
    styled_content: Vec<StyledContent<String>>,
}

impl StyledContentGroup {
    pub fn new(styled_content: Vec<StyledContent<String>>) -> Self {
        Self { styled_content }
    }

    pub fn len(&self) -> usize {
        self.styled_content.iter().fold(0, |acc, styled_element| {
            acc + styled_element.content().len()
        })
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.styled_content.is_empty()
    }

    pub fn join(&self, delimiter: StyledContent<String>) -> Self {
        let delimited = intersperse(self.styled_content.iter().cloned(), delimiter);

        StyledContentGroup::new(delimited.collect())
    }

    pub fn push(&mut self, element: StyledContent<String>) {
        self.styled_content.push(element);
    }
}

impl Display for StyledContentGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content_as_string = self
            .styled_content
            .iter()
            .fold(String::new(), |acc, element| acc + &element.to_string());
        write!(f, "{content_as_string}")
    }
}
