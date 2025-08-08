use anyhow::Result;
use crossterm::{
    ExecutableCommand, cursor as crossterm_cursor, event::Event as CrosstermTerminalEvent,
    terminal as crossterm_terminal,
};
use std::io::Write;

use crate::{
    commands::{CommandOutcome, LiveCommand},
    history,
    terminal::traits::{self as terminal_traits, KeyHandling},
};

const SPACE: &str = " ";

pub struct PleaseTerminal {
    live_command: LiveCommand,
    history: history::History,

    cursor_position: usize,
    history_pattern_position: usize,
}

impl terminal_traits::KeyHandling for PleaseTerminal {
    fn handle_enter(&mut self, stdout: &mut std::io::Stdout) -> CommandOutcome {
        let attempted_command = self.live_command.user_command_as_string();

        println!();
        if let Err(e) = stdout.flush() {
            log::error!("failed to flush newline before executing user command, error: {e}");
            return CommandOutcome::Continue;
        };

        let command_execution_result = self.live_command.execute_user_command();

        self.cursor_position = 0;
        self.history_pattern_position = 0;
        self.history.reset_history_search_index();

        let command_outcome = match command_execution_result {
            Ok(CommandOutcome::Close) => {
                self.history.add_command_to_cache(attempted_command);
                return CommandOutcome::Close;
            }
            Ok(command_outcome) => command_outcome,
            Err(e) => {
                log::error!("failed to execute command \"{attempted_command}\", error: {e}");
                CommandOutcome::Continue
            }
        };

        if !matches!(command_outcome, CommandOutcome::Skip) {
            self.history.add_command_to_cache(attempted_command);
        }

        print!("{}", self.live_command.live_command_prefix());
        if let Err(e) = stdout.flush() {
            log::error!("failed to flush command preffix after executing user command, error: {e}");
        };

        command_outcome
    }

    fn handle_backspace(
        &mut self,
        stdout: &mut std::io::Stdout,
        key_event: crossterm::event::KeyEvent,
    ) -> Result<()> {
        if self.cursor_position == 0 || self.live_command.user_command.is_empty() {
            return Ok(());
        }

        stdout.execute(crossterm_cursor::DisableBlinking)?;

        let steps = if key_event
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            self.calc_steps_to_previous_delimiter()
        } else {
            1
        };

        for _ in 0..steps {
            self.live_command
                .user_command
                .remove(self.cursor_position.saturating_sub(1));

            self.move_cursor_left(stdout, 1)?;
        }

        print!("{}", SPACE.repeat(steps));
        self.cursor_position += steps;
        self.move_cursor_left(stdout, steps)?;

        // not sure why but if the backspace is not from at the end, we need an extra backspace
        let early_position = self.cursor_position;

        self.write_command_suffix(stdout)?;

        if early_position != self.live_command.user_command.len() {
            self.move_cursor_left(stdout, 1)?;
        }

        self.history_pattern_position = self.cursor_position;
        self.history.reset_history_search_index();

        stdout.execute(crossterm_cursor::EnableBlinking)?;

        Ok(())
    }

    fn handle_up(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        self.handle_history_search(stdout, history::Direction::Previous)
    }

    fn handle_down(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        self.handle_history_search(stdout, history::Direction::Next)
    }

    fn handle_left(
        &mut self,
        stdout: &mut std::io::Stdout,
        key_event: crossterm::event::KeyEvent,
    ) -> Result<()> {
        let steps = if key_event
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            self.calc_steps_to_previous_delimiter()
        } else {
            1
        };

        self.move_cursor_left(stdout, steps)
    }

    fn handle_right(
        &mut self,
        stdout: &mut std::io::Stdout,
        key_event: crossterm::event::KeyEvent,
    ) -> Result<()> {
        let steps = if key_event
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            let command_len = self.live_command.user_command.len();

            // getting current cursor but not exceeding command len
            let start_index = self.cursor_position.min(command_len.saturating_sub(1));

            self.live_command.user_command[start_index..]
                .iter()
                .position(|c| !c.is_alphanumeric())
                // adding 1 to cross the delimiter
                .map(|index| index + 1)
                .unwrap_or_else(|| self.live_command.user_command.len() - self.cursor_position)
        } else {
            1
        };

        self.move_cursor_right(stdout, steps)
    }
}

impl PleaseTerminal {
    pub fn new(history: history::History) -> Self {
        Self {
            live_command: LiveCommand::new(),
            history,
            cursor_position: 0,
            history_pattern_position: 0,
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

                    let key_char = key_event.code.as_char();
                    if let Some(new_char) = key_char
                        && key_event.modifiers.is_empty()
                    {
                        self.handle_char_added(&mut stdout, new_char)?;
                    } else if key_event.code.is_enter() {
                        if let CommandOutcome::Close = self.handle_enter(&mut stdout) {
                            break;
                        };
                    } else if self.is_backspace_key_event(key_event) {
                        self.handle_backspace(&mut stdout, key_event)?;
                    } else if key_event.code.is_up() {
                        self.handle_up(&mut stdout)?;
                    } else if key_event.code.is_down() {
                        self.handle_down(&mut stdout)?;
                    } else if key_event.code.is_left() {
                        self.handle_left(&mut stdout, key_event)?;
                    } else if key_event.code.is_right() {
                        self.handle_right(&mut stdout, key_event)?;
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

    fn handle_char_added(&mut self, stdout: &mut std::io::Stdout, new_char: char) -> Result<()> {
        self.live_command
            .user_command
            .insert(self.cursor_position, new_char);

        stdout.execute(crossterm_cursor::DisableBlinking)?;
        self.write_command_suffix(stdout)?;
        stdout.execute(crossterm_cursor::EnableBlinking)?;

        self.history_pattern_position = self.cursor_position;
        self.history.reset_history_search_index();
        Ok(())
    }

    fn write_command_suffix(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        let suffix = &self.live_command.user_command_as_string()[self.cursor_position..];
        if !suffix.is_empty() {
            print!("{suffix}");
            stdout.flush()?;
        }

        stdout.execute(crossterm_terminal::Clear(
            crossterm_terminal::ClearType::FromCursorDown,
        ))?;

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

    fn handle_history_search(
        &mut self,
        stdout: &mut std::io::Stdout,
        direction: history::Direction,
    ) -> Result<()> {
        let current_command_string = self.live_command.user_command_as_string();
        let current_history_pattern = &current_command_string[..self.history_pattern_position];

        let new_command = self.get_new_command_from_history(direction, current_history_pattern);

        if let Some(fitting_command) = new_command {
            self.live_command.user_command = fitting_command.chars().collect();

            stdout.execute(crossterm_cursor::DisableBlinking)?;
            self.move_cursor_left(
                stdout,
                current_command_string
                    .len()
                    .saturating_sub(current_history_pattern.len()),
            )?;
            self.write_command_suffix(stdout)?;
            self.move_cursor_right(
                stdout,
                self.live_command
                    .user_command
                    .len()
                    .saturating_sub(self.cursor_position),
            )?;

            stdout.execute(crossterm_cursor::EnableBlinking)?;
        }

        stdout.execute(crossterm_cursor::EnableBlinking)?;

        Ok(())
    }

    fn get_new_command_from_history(
        &mut self,
        direction: history::Direction,
        current_history_pattern: &str,
    ) -> Option<String> {
        let historical_command_option = match direction {
            history::Direction::Previous => {
                self.history.navigate_to_previous(current_history_pattern)
            }
            history::Direction::Next => self.history.navigate_to_next(current_history_pattern),
        };

        let fitting_command =
            historical_command_option.map(|previous_command| previous_command.to_string());

        if let Some(fitting_command) = fitting_command {
            Some(fitting_command)
        } else if self.cursor_position != self.history_pattern_position {
            Some(current_history_pattern.to_string())
        } else {
            None
        }
    }

    fn is_backspace_key_event(&self, key_event: crossterm::event::KeyEvent) -> bool {
        key_event.code.is_backspace()
            || key_event.code.as_char().is_some_and(|c| {
                c == 'w'
                    && key_event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
            })
    }

    fn calc_steps_to_previous_delimiter(&self) -> usize {
        self.live_command.user_command[..self.cursor_position.saturating_sub(1)]
            .iter()
            .rev()
            .position(|c| !c.is_alphanumeric())
            // adding 1 to cross delimiter
            .map(|index| index + 1)
            .unwrap_or(self.cursor_position)
    }
}
