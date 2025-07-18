use anyhow::{Ok, Result};
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

        loop {
            let event = crossterm::event::read()?;
            match event {
                CrosstermTerminalEvent::Key(key_event) => {
                    let as_char = key_event.code.as_char();
                    if !key_event.is_press() {
                        continue;
                    }
                    if let Some(c) = as_char {
                        self.live_command.user_command.push(c);
                        print!("{c}");
                        stdout.flush()?;
                    } else if key_event.code.is_enter() {
                        if let CommandOutcome::Close = self.live_command.execute_user_command()? {
                            return Ok(());
                        };
                    } else if key_event.code.is_backspace() {
                        self.live_command.user_command.pop();

                        let (x, y) = crossterm_cursor::position()?;
                        if x == 0 && y == 0 {
                            continue;
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

                        stdout.flush()?;
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
}
