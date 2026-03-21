use anyhow::{Context, Result};
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor as crossterm_cursor,
    event::Event as CrosstermTerminalEvent, terminal as crossterm_terminal,
};
use std::io::Write;

use crate::{
    commands::{
        CommandOutcome, LiveCommand, completion::get_completion_provider, traits::ConcatType,
    },
    history,
    terminal::{
        tab_context::{self, TabResult},
        traits::{self as terminal_traits, IsKeyEvents, KeyHandling},
        types::EditEvent,
    },
    utils,
};

enum TerminalLoopEvent {
    Continue,
    Exit,
}

pub struct PleaseTerminal {
    live_command: LiveCommand,
    history: history::History,

    cursor_position: usize,
    history_pattern_position: usize,
}

impl terminal_traits::KeyHandling for PleaseTerminal {
    fn handle_enter(&mut self, stdout: &mut std::io::Stdout) -> CommandOutcome {
        let attempted_command = self.live_command.user_command_as_string();

        if let Err(e) = self.handle_end(stdout) {
            log::warn!("failed moving to end before executing command, error: {e}")
        };

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
            log::error!("failed to flush command prefix after executing user command, error: {e}");
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

        stdout.execute(crossterm_cursor::Hide)?;

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

        stdout.execute(crossterm_terminal::Clear(
            crossterm_terminal::ClearType::FromCursorDown,
        ))?;

        self.write_command_suffix(stdout, EditEvent::Deletion)?;

        self.history_pattern_position = self.cursor_position;
        self.history.reset_history_search_index();

        stdout.execute(crossterm_cursor::Show)?;

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

    fn handle_ctrl_c(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        self.move_cursor_right(
            stdout,
            self.live_command
                .user_command
                .len()
                .saturating_sub(self.cursor_position)
                .min(0),
        )?;
        print!("^C");
        stdout
            .queue(crossterm_terminal::Clear(
                crossterm_terminal::ClearType::FromCursorDown,
            ))?
            .flush()?;

        self.live_command.user_command.clear();
        self.handle_enter(stdout);
        Ok(())
    }

    fn handle_home(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        self.move_cursor_left(stdout, self.cursor_position)
    }

    fn handle_end(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        let steps_right = self
            .live_command
            .user_command_as_string()
            .len()
            .saturating_sub(self.cursor_position);

        self.move_cursor_right(stdout, steps_right)
    }

    fn handle_tab(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        let command_string = self.live_command.user_command_as_string();

        let completion_provider = get_completion_provider(&command_string);
        let possible_completions = completion_provider.try_completing(&command_string)?;

        if possible_completions.is_empty() {
            return Ok(());
        } else if possible_completions.len() == 1 {
            let addition = &possible_completions[0];
            match &addition.concat_type {
                ConcatType::Delimited(concat_string) => {
                    if !command_string.is_empty() {
                        self.handle_string_added(stdout, concat_string)?;
                    }

                    self.handle_string_added(stdout, &addition.value)?;
                }
                ConcatType::PrefixConcat(start_index) => {
                    self.handle_string_added(stdout, &addition.value[start_index.to_owned()..])?;
                }
            }

            stdout.flush()?;

            return Ok(());
        }

        // moving the cursor to end of command to allow appending completion later on
        self.handle_end(stdout)?;

        let latest_word = self.live_command.get_latest_word();
        let mut tab_context_runner =
            tab_context::TabContext::new(&possible_completions, &latest_word, stdout)
                .context("failed creating tab context")?;

        let tab_outcome = tab_context_runner.run()?;

        match tab_outcome {
            TabResult::AppendText(text) => {
                self.handle_string_added(stdout, &text)?;
            }
            TabResult::KeyEvent(key_event) => {
                self.handle_event(stdout, CrosstermTerminalEvent::Key(key_event))?;
            }
            TabResult::None => {}
        }

        Ok(())
    }
}

impl terminal_traits::IsKeyEvents for PleaseTerminal {
    fn is_backspace_key_event(&self, key_event: crossterm::event::KeyEvent) -> bool {
        key_event.code.is_backspace()
            || key_event.code.as_char().is_some_and(|c| {
                c == 'w'
                    && key_event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
            })
    }

    fn is_ctrl_c_key_event(&self, key_event: crossterm::event::KeyEvent) -> bool {
        key_event.code.as_char().is_some_and(|c| {
            c == 'c'
                && key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
        })
    }
}

impl PleaseTerminal {
    pub fn new(history: history::History, command_manager: LiveCommand) -> Self {
        Self {
            live_command: command_manager,
            history,
            cursor_position: 0,
            history_pattern_position: 0,
        }
    }

    fn handle_event(
        &mut self,
        stdout: &mut std::io::Stdout,
        event: CrosstermTerminalEvent,
    ) -> Result<TerminalLoopEvent> {
        match event {
            CrosstermTerminalEvent::Key(key_event) => {
                if !key_event.is_press() {
                    return Ok(TerminalLoopEvent::Continue);
                }

                if let Some(new_char) = self.get_char_from_key_event(key_event) {
                    self.handle_char_added(stdout, new_char)?;
                } else if key_event.code.is_enter() {
                    if let CommandOutcome::Close = self.handle_enter(stdout) {
                        return Ok(TerminalLoopEvent::Exit);
                    };
                } else if self.is_backspace_key_event(key_event) {
                    self.handle_backspace(stdout, key_event)?;
                } else if self.is_ctrl_c_key_event(key_event) {
                    self.handle_ctrl_c(stdout)?;
                } else if key_event.code.is_tab() {
                    self.handle_tab(stdout)?;
                } else if key_event.code.is_up() {
                    self.handle_up(stdout)?;
                } else if key_event.code.is_down() {
                    self.handle_down(stdout)?;
                } else if key_event.code.is_left() {
                    self.handle_left(stdout, key_event)?;
                } else if key_event.code.is_right() {
                    self.handle_right(stdout, key_event)?;
                } else if key_event.code.is_home() {
                    self.handle_home(stdout)?;
                } else if key_event.code.is_end() {
                    self.handle_end(stdout)?;
                }

                Ok(TerminalLoopEvent::Continue)
            }
            CrosstermTerminalEvent::FocusGained
            | CrosstermTerminalEvent::FocusLost
            | CrosstermTerminalEvent::Resize(_, _) => Ok(TerminalLoopEvent::Continue),
            CrosstermTerminalEvent::Mouse(_) => todo!(),
            CrosstermTerminalEvent::Paste(_) => todo!(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut stdout = std::io::stdout();

        print!("{}", self.live_command.live_command_prefix());
        stdout.flush()?;

        loop {
            let event = crossterm::event::read()?;
            if let TerminalLoopEvent::Exit = self.handle_event(&mut stdout, event)? {
                break;
            }
        }

        self.history.save_history_to_persistent_file()?;

        Ok(())
    }

    fn handle_char_added(&mut self, stdout: &mut std::io::Stdout, new_char: char) -> Result<()> {
        self.live_command
            .user_command
            .insert(self.cursor_position, new_char);

        stdout.execute(crossterm_cursor::Hide)?;
        self.write_command_suffix(stdout, EditEvent::Addition)?;
        stdout.execute(crossterm_cursor::Show)?;

        self.history_pattern_position = self.cursor_position;
        self.history.reset_history_search_index();
        Ok(())
    }

    fn handle_string_added(
        &mut self,
        stdout: &mut std::io::Stdout,
        new_string: &str,
    ) -> Result<()> {
        for c in new_string.chars() {
            self.handle_char_added(stdout, c)?;
        }

        Ok(())
    }

    fn write_command_suffix(
        &mut self,
        stdout: &mut std::io::Stdout,
        edit_event: EditEvent,
    ) -> Result<()> {
        let initial_position = crossterm_cursor::position()?;

        let suffix = &self.live_command.user_command_as_string()[self.cursor_position..];
        if !suffix.is_empty() {
            print!("{suffix}");
            stdout.flush()?;
        }

        let cursor_position_x: usize = crossterm_cursor::position()
            .context("failed to get cursor position to write command suffix")?
            .0
            .into();
        let terminal_size_x: usize = crossterm_terminal::size()
            .context("failed to get terminal to write command suffix")?
            .0
            .into();

        if edit_event == EditEvent::Deletion {
            if cursor_position_x + 1 != terminal_size_x {
                stdout.execute(crossterm_terminal::Clear(
                    crossterm_terminal::ClearType::FromCursorDown,
                ))?;

                for _ in 0..suffix.len() {
                    utils::move_left(stdout)?;
                }
            } else {
                let full_len = self.live_command.get_full_len();
                let mut steps = self
                    .live_command
                    .user_command
                    .len()
                    .saturating_sub(self.cursor_position);
                if full_len == terminal_size_x {
                    print!(" ");
                    stdout.flush()?;
                    steps = steps.saturating_add(1);
                }

                // 12345678 -> 1234567 -> 123456
                // 12345678 -> 1234568 -> 123458
                // 1234567  -> 123456  -> 12345
                // 1234567  -> 123457  -> 12347

                for _ in 0..steps {
                    utils::move_left(stdout)?;
                }
            }

            return Ok(());
        }

        self.cursor_position = self.live_command.user_command.len();
        let mut left_steps = suffix.len().saturating_sub(1);

        if cursor_position_x + 1 != terminal_size_x || edit_event == EditEvent::History {
            stdout.execute(crossterm_terminal::Clear(
                crossterm_terminal::ClearType::FromCursorDown,
            ))?;
        } else if initial_position.0 as usize + suffix.len() == terminal_size_x as usize {
            // handle edge case where at end of line and still staying instead of moving to next
            print!(" ");
            stdout.flush()?;
            left_steps += 1;
            self.cursor_position += 1;
        }

        // since the print sends the actual cursor to the end, we rewind it
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

            utils::move_left(stdout)?;
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

            stdout.execute(crossterm_cursor::Hide)?;
            self.move_cursor_left(
                stdout,
                current_command_string
                    .len()
                    .saturating_sub(current_history_pattern.len()),
            )?;
            self.write_command_suffix(stdout, EditEvent::History)?;
            self.move_cursor_right(
                stdout,
                self.live_command
                    .user_command
                    .len()
                    .saturating_sub(self.cursor_position),
            )?;
        }

        stdout.execute(crossterm_cursor::Show)?;

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

    fn get_char_from_key_event(&self, key_event: crossterm::event::KeyEvent) -> Option<char> {
        if let Some(new_char) = key_event.code.as_char()
            && !key_event
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            Some(new_char)
        } else {
            None
        }
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
