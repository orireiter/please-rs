use anyhow::{Context, Result};
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor as crossterm_cursor,
    event::{Event as CrosstermTerminalEvent, KeyCode as CrosstermTerminalKeyCode},
    terminal as crossterm_terminal,
};
use std::io::Write;

use crate::terminal::{Action, ActionType};
use crate::{
    commands::{CommandOutcome, LiveCommand},
    history,
};

pub struct PleaseTerminal {
    live_command: LiveCommand,
    history: history::History,
    history_pattern_match_max_index: usize,

    cursor_position: usize,
    _history_pattern_position: usize,
}

impl PleaseTerminal {
    pub fn new(history: history::History) -> Self {
        Self {
            live_command: LiveCommand::new(),
            history,
            history_pattern_match_max_index: 0,
            cursor_position: 0,
            _history_pattern_position: 0,
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
                        self.handle_backspace(&mut stdout, &Action::new_user_action(event))?
                    } else if key_event.code.is_up() {
                        self.handle_up_pressed(&mut stdout)?
                    } else if key_event.code.is_down() {
                        self.handle_down_pressed(&mut stdout)?
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
        if let ActionType::UserAction = action.action_type {
            self.history_pattern_match_max_index = self.live_command.user_command.len();
            self.history.reset_history_search_index();
        }

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
        self.history_pattern_match_max_index = 0;
        self.history.reset_history_search_index();

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

    fn handle_backspace(&mut self, stdout: &mut std::io::Stdout, action: &Action) -> Result<()> {
        if self.live_command.user_command.is_empty() {
            return Ok(());
        }

        self.live_command.user_command.pop();
        if let ActionType::UserAction = action.action_type {
            self.history_pattern_match_max_index = self.live_command.user_command.len();
            self.history.reset_history_search_index();
        }

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

    // todo optimize and not backspace everything just to re-write
    fn handle_up_pressed(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        let current_command_string = self.live_command.user_command_as_string();
        let previous_fitting_command = self
            .history
            .navigate_to_previous(&current_command_string[..self.history_pattern_match_max_index])
            .map(|previous_command| previous_command.to_string());

        if let Some(previous_fitting_command) = previous_fitting_command {
            self.override_current_command(stdout, current_command_string, previous_fitting_command)?
        }

        Ok(())
    }

    fn handle_down_pressed(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        let current_command_string = self.live_command.user_command_as_string();
        let next_fitting_command = self
            .history
            .navigate_to_next(&current_command_string[..self.history_pattern_match_max_index])
            .map(|previous_command| previous_command.to_string());

        if let Some(next_fitting_command) = next_fitting_command {
            self.override_current_command(stdout, current_command_string, next_fitting_command)?
        }

        Ok(())
    }

    fn override_current_command(
        &mut self,
        stdout: &mut std::io::Stdout,
        current_command: String,
        new_command: String,
    ) -> Result<()> {
        if current_command != new_command {
            stdout.execute(crossterm_cursor::DisableBlinking)?;

            let backspace_amount = current_command
                .len()
                .saturating_sub(self.history_pattern_match_max_index);
            let backspace_action =
                Action::new_history_action_by_key_code(CrosstermTerminalKeyCode::Backspace);
            for _ in 0..backspace_amount {
                self.handle_backspace(stdout, &backspace_action)?;
            }

            let left_to_write = &new_command[self.history_pattern_match_max_index..];
            for ch in left_to_write.chars() {
                self.handle_char_added(stdout, Action::new_history_key_pressed_action(ch))?;
            }

            stdout.execute(crossterm_cursor::EnableBlinking)?;
        }

        Ok(())
    }
}

#[allow(dead_code)]
impl PleaseTerminal {
    fn handle_char_added_v2(&mut self, stdout: &mut std::io::Stdout, new_char: char) -> Result<()> {
        self.live_command
            .user_command
            .insert(self.cursor_position, new_char);

        stdout.execute(crossterm_cursor::DisableBlinking)?;
        self.write_command_suffix(stdout)?;
        stdout.execute(crossterm_cursor::EnableBlinking)?;

        // self.history_pattern_position = self.live_command.user_command.len();
        // self.history.reset_history_search_index();
        Ok(())
    }

    fn handle_backspace_v2(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        if self.cursor_position == 0 || self.live_command.user_command.is_empty() {
            return Ok(());
        }

        stdout.execute(crossterm_cursor::DisableBlinking)?;

        self.live_command
            .user_command
            .remove(self.cursor_position.saturating_sub(1));

        self.move_cursor_left(stdout, 1)?;
        print!(" ");
        self.cursor_position += 1;
        self.move_cursor_left(stdout, 1)?;

        // not sure why but if the backspace is not from at the end, we need an extra backspace
        let early_position = self.cursor_position;

        self.write_command_suffix(stdout)?;

        if early_position != self.live_command.user_command.len() {
            self.move_cursor_left(stdout, 1)?;
        }

        stdout.execute(crossterm_cursor::EnableBlinking)?;

        Ok(())
    }

    fn write_command_suffix(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        let suffix = &self.live_command.user_command_as_string()[self.cursor_position..];
        if suffix.is_empty() {
            return Ok(());
        }

        print!("{suffix}");
        stdout.flush()?;

        if self.cursor_position + 1 != self.live_command.user_command.len() {
            stdout.execute(crossterm_terminal::Clear(
                crossterm_terminal::ClearType::FromCursorDown,
            ))?;
        }

        // since the print sends the actual cursor to the end, we rewind it
        self.cursor_position = self.live_command.user_command.len();
        let left_steps = suffix.len().saturating_sub(1);
        self.move_cursor_left(stdout, left_steps)?;

        Ok(())
    }

    fn move_cursor_right(&mut self, stdout: &mut std::io::Stdout, steps: usize) -> Result<()> {
        for _ in 0..steps {
            if self.cursor_position == self.live_command.user_command.len() {
                break;
            }

            let (x, y) = crossterm_cursor::position()?;
            let terminal_size = crossterm_terminal::size()?;
            if x + 1 < terminal_size.0 {
                stdout.execute(crossterm_cursor::MoveRight(1))?;
            } else {
                stdout.execute(crossterm_cursor::MoveTo(0, y + 1))?;
            }

            self.cursor_position += 1;
        }

        Ok(())
    }

    fn move_cursor_left(&mut self, stdout: &mut std::io::Stdout, steps: usize) -> Result<()> {
        for _ in 0..steps {
            if self.cursor_position == 0 {
                break;
            }

            let (x, y) = crossterm_cursor::position()?;
            if x == 0 && y == 0 {
                break;
            } else if x > 0 {
                stdout.execute(crossterm_cursor::MoveLeft(1))?;
            } else {
                let terminal_size = crossterm_terminal::size()?;
                stdout.execute(crossterm_cursor::MoveTo(terminal_size.0, y - 1))?;
            }
            self.cursor_position = self.cursor_position.saturating_sub(1);
        }

        Ok(())
    }
}
