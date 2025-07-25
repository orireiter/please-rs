use crate::terminal::Action;
use anyhow::{Context, Result};
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor as crossterm_cursor,
    event::Event as CrosstermTerminalEvent, terminal as crossterm_terminal,
};
use std::io::Write;

use crate::{
    commands::{CommandOutcome, LiveCommand},
    history,
};

pub struct PleaseTerminal {
    live_command: LiveCommand,
    history: history::History,
}

impl PleaseTerminal {
    pub fn new(history: history::History) -> Self {
        Self {
            live_command: LiveCommand::new(),
            history,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut stdout = std::io::stdout();

        print!("{}", self.live_command.live_command_prefix());
        stdout.flush()?;

        loop {
            let event = crossterm::event::read()?;
            match event {
                CrosstermTerminalEvent::Key(key_event) => {
                    if !key_event.is_press() {
                        continue;
                    }

                    if key_event.code.as_char().is_some() {
                        self.handle_char_added(&mut stdout, Action::new_user_action(event))?;
                    } else if key_event.code.is_enter() {
                        if let CommandOutcome::Close = self.handle_enter_pressed(&mut stdout) {
                            break;
                        };
                    } else if key_event.code.is_backspace() {
                        self.handle_backspace(&mut stdout)?
                    } else if key_event.code.is_up() {
                        self.handle_up_pressed(&mut stdout)?
                    }
                }
                CrosstermTerminalEvent::FocusGained => {}
                CrosstermTerminalEvent::FocusLost => {}
                CrosstermTerminalEvent::Mouse(_) => todo!(),
                CrosstermTerminalEvent::Paste(_) => todo!(),
                CrosstermTerminalEvent::Resize(_, _) => {}
            }
        }

        self.history.save_history_to_persistent_file()?;

        Ok(())
    }

    fn handle_char_added(&mut self, stdout: &mut std::io::Stdout, action: Action) -> Result<()> {
        let CrosstermTerminalEvent::Key(key_event) = action.event else {
            return Err(anyhow::anyhow!(
                "handle char function got non-key event {:?}",
                action.event
            ));
        };

        let Some(c) = key_event.code.as_char() else {
            return Err(anyhow::anyhow!(
                "handle char function got non-char key event {:?}",
                key_event.code
            ));
        };

        self.live_command.user_command.push(c);

        self.history.reset_history_search_index();

        print!("{c}");
        stdout
            .flush()
            .context(format!("failed to flush after adding char {c}"))
    }

    fn handle_enter_pressed(&mut self, stdout: &mut std::io::Stdout) -> CommandOutcome {
        let attempted_command = self.live_command.user_command_as_string();

        self.history.add_command_to_cache(attempted_command.clone());

        println!();
        if let Err(e) = stdout.flush() {
            log::error!("failed to flush newline before executing user command, error: {e}");
            return CommandOutcome::Continue;
        };

        let command_execution_result = self.live_command.execute_user_command();

        let command_outcome = match command_execution_result {
            Ok(CommandOutcome::Close) => return CommandOutcome::Close,
            Ok(command_outcome) => command_outcome,
            Err(e) => {
                log::error!("failed to execute command \"{attempted_command}\", error: {e}");
                CommandOutcome::Continue
            }
        };

        print!("{}", self.live_command.live_command_prefix());
        if let Err(e) = stdout.flush() {
            log::error!("failed to flush command preffix after executing user command, error: {e}");
        };

        command_outcome
    }

    fn handle_backspace(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        if self.live_command.user_command.is_empty() {
            return Ok(());
        }

        self.history.reset_history_search_index();

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

    fn handle_up_pressed(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        let current_command_string = self.live_command.user_command_as_string();
        let previous_fitting_command = self
            .history
            .navigate_to_previous(&current_command_string)
            .map(|previous_command| previous_command.to_string());

        if let Some(previous_fitting_command) = previous_fitting_command
            && let Some(stripped_command) =
                previous_fitting_command.strip_prefix(&current_command_string)
        {
            stdout.execute(crossterm_cursor::DisableBlinking)?;

            for ch in stripped_command.chars() {
                self.handle_char_added(stdout, Action::new_history_key_pressed_action(ch))?;
            }

            stdout.execute(crossterm_cursor::EnableBlinking)?;
        }

        Ok(())
    }
}
