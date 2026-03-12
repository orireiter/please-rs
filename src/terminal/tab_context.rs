use std::io::Write;

use anyhow::{Ok, Result};
use crossterm::{
    cursor as crossterm_cursor, event::Event as CrosstermEvent, event::KeyEvent, style::Attribute,
    terminal as crossterm_terminal,
};

use crate::{
    commands::traits::CompletionCandidate,
    utils::{self, SPACE},
};

#[derive(Debug)]
#[allow(dead_code)]
pub enum TabResult<'a> {
    None,
    AppendText(&'a String),
    KeyEvent(KeyEvent),
}

pub struct TabContext<'a> {
    possible_completions: &'a Vec<CompletionCandidate>,
    current_selection_index: usize,
    stdout: &'a mut std::io::Stdout,
}

impl<'a> TabContext<'a> {
    pub fn new(
        possible_completions: &'a Vec<CompletionCandidate>,
        stdout: &'a mut std::io::Stdout,
    ) -> Self {
        Self {
            possible_completions,
            current_selection_index: 0,
            stdout,
        }
    }

    fn get_stylized_candidate(&self, candidate_index: usize) -> String {
        let candidate = &self.possible_completions[candidate_index].value;

        if self.current_selection_index == candidate_index {
            format!(
                "{}{}{}  ",
                Attribute::Reverse,
                candidate,
                Attribute::NoReverse
            )
        } else {
            format!("{}  ", candidate)
        }
    }

    fn setup_run(&mut self) -> Result<()> {
        let steps_to_eol = self.calc_steps_end_of_line()?;
        for _ in 0..steps_to_eol {
            print!("{SPACE}");
        }

        let mut cursor_advancement = 0;
        for i in 0..self.possible_completions.len() {
            let stylized_candidate = self.get_stylized_candidate(i);
            print!("{}", stylized_candidate);
            cursor_advancement += self.possible_completions[i].value.len() + 2;
        }

        self.stdout.flush()?;

        for _ in 0..(cursor_advancement + steps_to_eol) {
            utils::move_left(self.stdout)?;
        }

        Ok(())
    }

    pub fn run(&'_ mut self) -> Result<TabResult<'_>> {
        self.setup_run()?;

        loop {
            let event = crossterm::event::read()?;
            match event {
                CrosstermEvent::Key(key_event) => {
                    if !key_event.is_press() {
                        continue;
                    }

                    if key_event.code.is_up() {
                        todo!("handle tab up key");
                    } else if key_event.code.is_down() {
                        todo!("handle tab down key");
                    } else if key_event.code.is_left() {
                        self.handle_left();
                    } else if key_event.code.is_right() {
                        self.handle_right();
                    } else if key_event.code.is_enter() {
                        return Ok(TabResult::AppendText(
                            &self.possible_completions[self.current_selection_index].value,
                        ));
                    } else {
                        return Ok(TabResult::KeyEvent(key_event));
                    }

                    self.setup_run()?;
                }
                CrosstermEvent::FocusLost | CrosstermEvent::FocusGained => {
                    continue;
                }
                _ => todo!("{event:?}"),
            }
        }
    }

    fn calc_steps_end_of_line(&self) -> Result<usize> {
        let (cursor_x, _) = crossterm_cursor::position()?;
        let (terminal_size_x, _) = crossterm_terminal::size()?;

        Ok(terminal_size_x.saturating_sub(cursor_x).into())
    }

    fn handle_right(&mut self) {
        if self.current_selection_index != self.possible_completions.len() - 1 {
            self.current_selection_index += 1;
        }
    }

    fn handle_left(&mut self) {
        if self.current_selection_index > 0 {
            self.current_selection_index -= 1;
        }
    }
}
