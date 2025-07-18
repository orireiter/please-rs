use anyhow::{Context, Result};
use crossterm::{
    QueueableCommand, cursor as crossterm_cursor, event::Event as CrosstermTerminalEvent,
    terminal as crossterm_terminal,
};
use std::io::Write;

use crate::commands::{CommandOutcome, LiveCommand};

pub struct PleaseTerminal {
    live_command: LiveCommand,
}

impl PleaseTerminal {
    pub fn new() -> Self {
        Self {
            live_command: LiveCommand::new(),
        }
    }

    pub fn start(mut self) -> Result<()> {
        let mut stdout = std::io::stdout();

        print!("{}", self.live_command.live_command_prefix());
        stdout.flush()?;

        loop {
            let event = crossterm::event::read()?;
            match event {
                CrosstermTerminalEvent::Key(key_event) => {
                    let as_char = key_event.code.as_char();
                    if !key_event.is_press() {
                        continue;
                    }
                    if let Some(c) = as_char {
                        self.handle_char_added(&mut stdout, c)?;
                    } else if key_event.code.is_enter() {
                        if let CommandOutcome::Close = self.handle_enter_pressed() {
                            return Ok(());
                        };
                    } else if key_event.code.is_backspace() {
                        self.handle_backspace(&mut stdout)?
                    }
                }
                CrosstermTerminalEvent::FocusGained => {}
                CrosstermTerminalEvent::FocusLost => {}
                CrosstermTerminalEvent::Mouse(_) => todo!(),
                CrosstermTerminalEvent::Paste(_) => todo!(),
                CrosstermTerminalEvent::Resize(_, _) => {}
            }
        }
    }

    fn handle_char_added(&mut self, stdout: &mut std::io::Stdout, c: char) -> Result<()> {
        self.live_command.user_command.push(c);
        print!("{c}");
        stdout
            .flush()
            .context(format!("failed to flush after adding char {c}"))
    }

    fn handle_enter_pressed(&mut self) -> CommandOutcome {
        match self.live_command.execute_user_command() {
            Ok(command_outcome) => command_outcome,
            Err(e) => {
                log::error!(
                    "failed to execute command {}, error: {}",
                    self.live_command.user_command_as_string(),
                    e
                );
                CommandOutcome::Continue
            }
        }
    }

    fn handle_backspace(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        self.live_command.user_command.pop();

        let (x, y) = crossterm_cursor::position()?;
        if x == 0 && y == 0 {
            return Ok(());
        } else if x > 0 {
            stdout.queue(crossterm_cursor::MoveLeft(1))?;
            print!(" ");
            stdout.queue(crossterm_cursor::MoveLeft(1))?;
        } else {
            let terminal_size = crossterm_terminal::size()?;
            stdout.queue(crossterm_cursor::MoveTo(terminal_size.0, y - 1))?;
            print!(" ");
            stdout.queue(crossterm_cursor::MoveTo(terminal_size.0, y - 1))?;
        }

        stdout.flush().context("failed to flush after backspace")
    }
}
