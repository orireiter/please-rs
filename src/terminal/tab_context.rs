use std::io::Write;

use anyhow::{Context, Result};
use crossterm::{
    ExecutableCommand, QueueableCommand, cursor as crossterm_cursor,
    event::{Event as CrosstermEvent, KeyEvent},
    style::Attribute,
    terminal as crossterm_terminal,
};

use crate::commands::traits::{CompletionCandidate, ConcatType};

#[derive(Debug)]
pub enum TabResult {
    AppendText(String),
    KeyEvent(KeyEvent),
}

#[allow(dead_code)]
struct CandidatesGridConfig {
    starting_indices: Vec<usize>,
}

impl<'a> CandidatesGridConfig {
    const _DEFAULT_MINIMUM_SPACE: usize = 2;

    #[allow(dead_code)]
    pub fn new(
        _terminal_size: (u16, u16),
        _candidates: &'a Vec<CompletionCandidate>,
        _minimum_space: Option<usize>,
    ) -> Self {
        todo!()
    }
}

pub struct TabContext<'a> {
    possible_completions: &'a Vec<CompletionCandidate>,
    candidates_grid_config: CandidatesGridConfig,
    current_selection_index: usize,
    current_live_command_latest_arg: &'a str,
    stdout: &'a mut std::io::Stdout,
}

impl<'a> TabContext<'a> {
    pub fn new(
        possible_completions: &'a Vec<CompletionCandidate>,
        current_live_command_latest_arg: &'a str,
        stdout: &'a mut std::io::Stdout,
    ) -> Result<Self> {
        let candidates_grid_config = Self::get_candidate_grid_config(possible_completions)?;

        Ok(Self {
            possible_completions,
            candidates_grid_config,
            current_selection_index: 0,
            current_live_command_latest_arg,
            stdout,
        })
    }

    fn get_candidate_grid_config(
        possible_completions: &Vec<CompletionCandidate>,
    ) -> Result<CandidatesGridConfig> {
        let terminal_size = crossterm_terminal::size()?;
        Ok(CandidatesGridConfig::new(
            terminal_size,
            possible_completions,
            None,
        ))
    }

    fn get_stylized_candidate(&self, candidate_index: usize) -> String {
        let candidate = &self.possible_completions[candidate_index].value;

        if self.current_selection_index == candidate_index {
            format!(
                "{}{}{}",
                Attribute::Reverse,
                candidate,
                Attribute::NoReverse
            )
        } else {
            candidate.to_string()
        }
    }

    fn setup(&mut self) -> Result<()> {
        let position = crossterm_cursor::position()?;
        let mut lines_down = 1;

        self.stdout
            .queue(crossterm_cursor::Hide)?
            .queue(crossterm_cursor::MoveToNextLine(1))?;

        let mut current_index = 0;
        for chunk in self
            .possible_completions
            .chunks(self.candidates_grid_config.starting_indices.len())
        {
            for (_, position_index) in chunk
                .iter()
                .zip(0..self.candidates_grid_config.starting_indices.len())
            {
                let new_index =
                    u16::try_from(self.candidates_grid_config.starting_indices[position_index])?;
                if new_index > 0 {
                    self.stdout
                        .queue(crossterm_cursor::MoveToColumn(new_index))?;
                }

                let stylized_candidate = self.get_stylized_candidate(current_index);
                print!("{}", stylized_candidate); // todo take in account that maybe an item dropped line

                current_index += 1;
            }

            self.stdout.queue(crossterm_cursor::MoveToNextLine(1))?;
            lines_down += 1;
        }

        self.stdout
            .queue(crossterm_cursor::MoveToPreviousLine(lines_down))?
            .queue(crossterm_cursor::MoveToColumn(position.0))?
            .queue(crossterm_cursor::Show)?
            .flush()?;

        Ok(())
    }

    fn update_selection_style(&mut self, previous_index: usize, new_index: usize) -> Result<()> {
        self.stdout.queue(crossterm_cursor::SavePosition)?;

        let (_, y) = crossterm_cursor::position()?;
        let items_per_row = self.candidates_grid_config.starting_indices.len();
        let rows_down: u16 = ((previous_index / items_per_row) + 1).try_into()?;
        let previous_index_y = y + rows_down;

        let start_index = previous_index % items_per_row;
        let previous_index_x =
            self.candidates_grid_config.starting_indices[start_index].try_into()?;

        self.stdout
            .queue(crossterm_cursor::MoveTo(previous_index_x, previous_index_y))?;
        print!("{}", self.get_stylized_candidate(previous_index));

        let items_per_row = self.candidates_grid_config.starting_indices.len();
        let rows_down: u16 = ((new_index / items_per_row) + 1).try_into()?;
        let new_index_y = y + rows_down;

        let start_index = new_index % items_per_row;
        let new_index_x = self.candidates_grid_config.starting_indices[start_index].try_into()?;

        self.stdout
            .queue(crossterm_cursor::MoveTo(new_index_x, new_index_y))?;
        print!("{}", self.get_stylized_candidate(new_index));

        self.stdout
            .queue(crossterm_cursor::RestorePosition)?
            .flush()?;
        Ok(())
    }

    fn teardown(&mut self) -> Result<()> {
        self.stdout
            .execute(crossterm_terminal::Clear(
                crossterm_terminal::ClearType::FromCursorDown,
            ))
            .context("failed to teardown tab context")?;
        Ok(())
    }

    fn run_loop(&mut self) -> Result<TabResult> {
        loop {
            let event = crossterm::event::read()?;
            match event {
                CrosstermEvent::Key(key_event) => {
                    if !key_event.is_press() {
                        continue;
                    }

                    if key_event.code.is_up() {
                        self.handle_up()?;
                    } else if key_event.code.is_down() {
                        self.handle_down()?;
                    } else if key_event.code.is_left() {
                        self.handle_left()?;
                    } else if key_event.code.is_right() {
                        self.handle_right()?;
                    } else if key_event.code.is_enter() {
                        return Ok(self.get_selected_tab_result());
                    } else {
                        return Ok(TabResult::KeyEvent(key_event));
                    } // todo specifically handle ctrl c?
                }
                CrosstermEvent::Resize(_, _) => {
                    self.teardown()?;
                    self.setup()?;
                }
                CrosstermEvent::FocusLost | CrosstermEvent::FocusGained => {
                    continue;
                }
                CrosstermEvent::Mouse(_) | CrosstermEvent::Paste(_) => todo!(),
            }
        }
    }

    pub fn run(&mut self) -> Result<TabResult> {
        self.setup()?;
        let result = self.run_loop();
        self.teardown()?;

        result
    }

    fn handle_right(&mut self) -> Result<()> {
        if self.current_selection_index != self.possible_completions.len() - 1 {
            self.current_selection_index += 1;

            self.update_selection_style(
                self.current_selection_index - 1,
                self.current_selection_index,
            )?;
        } else {
            self.current_selection_index = 0;

            self.update_selection_style(
                self.possible_completions.len() - 1,
                self.current_selection_index,
            )?;
        }

        Ok(())
    }

    fn handle_left(&mut self) -> Result<()> {
        if self.current_selection_index > 0 {
            self.current_selection_index -= 1;
            self.update_selection_style(
                self.current_selection_index + 1,
                self.current_selection_index,
            )?;
        } else {
            self.current_selection_index = self.possible_completions.len() - 1;

            self.update_selection_style(0, self.current_selection_index)?;
        }

        Ok(())
    }

    fn handle_up(&mut self) -> Result<()> {
        let row = self.current_selection_index / self.candidates_grid_config.starting_indices.len();

        if row != 0 {
            self.current_selection_index -= self.candidates_grid_config.starting_indices.len();
            self.update_selection_style(
                self.current_selection_index + self.candidates_grid_config.starting_indices.len(),
                self.current_selection_index,
            )?;
        }

        Ok(())
    }

    fn handle_down(&mut self) -> Result<()> {
        if self.current_selection_index
            < self.possible_completions.len() - self.candidates_grid_config.starting_indices.len()
        {
            self.current_selection_index += self.candidates_grid_config.starting_indices.len();
            self.update_selection_style(
                self.current_selection_index - self.candidates_grid_config.starting_indices.len(),
                self.current_selection_index,
            )?;
        }

        Ok(())
    }

    fn get_selected_tab_result(&self) -> TabResult {
        let selected_completion = &self.possible_completions[self.current_selection_index];

        if self.current_live_command_latest_arg.is_empty() {
            return TabResult::AppendText(selected_completion.value.clone());
        }

        match &selected_completion.concat_type {
            ConcatType::Delimited(delimiter) => {
                TabResult::AppendText(delimiter.to_owned() + &selected_completion.value.to_string())
            }
            ConcatType::PrefixConcat(start_index) => {
                let sliced_string = selected_completion.value[start_index.to_owned()..].to_string();
                TabResult::AppendText(sliced_string)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CandidatesGridConfig;
    use crate::commands::traits::{CompletionCandidate, ConcatType};

    fn candidates(values: &[&str]) -> Vec<CompletionCandidate> {
        values
            .iter()
            .map(|value| {
                CompletionCandidate::new(
                    (*value).to_string(),
                    ConcatType::Delimited(" ".to_string()),
                )
            })
            .collect()
    }

    #[test]
    fn grid_single_line_uses_all_candidates() {
        let candidates = candidates(&["abc", "de", "fghi"]);

        let config = CandidatesGridConfig::new((40, 20), &candidates, Some(2));

        // Expected printed grid (if this test passes):
        // abc  de  fghi

        assert_eq!(config.starting_indices, vec![0, 5, 9]);
    }

    #[test]
    fn grid_wraps_and_keeps_widest_column_widths() {
        let candidates = candidates(&["aa", "bb", "longer", "c"]);

        let config = CandidatesGridConfig::new((12, 20), &candidates, Some(2));

        // Expected printed grid (if this test passes):
        // aa      bb
        // longer  c

        assert_eq!(config.starting_indices, vec![0, 8]);
    }

    #[test]
    fn grid_respects_custom_minimum_spacing() {
        let candidates = candidates(&["a", "bb", "ccc"]);

        let config = CandidatesGridConfig::new((20, 20), &candidates, Some(4));

        // Expected printed grid (if this test passes):
        // a    bb    ccc

        assert_eq!(config.starting_indices, vec![0, 5, 11]);
    }

    #[test]
    fn grid_empty_candidates_returns_empty_layout() {
        let candidates = candidates(&[]);

        let config = CandidatesGridConfig::new((20, 20), &candidates, Some(2));

        // Expected printed grid (if this test passes):
        // [no rows printed]

        assert_eq!(config.starting_indices, vec![]);
    }

    #[test]
    fn grid_exact_terminal_fit_stays_single_line() {
        let candidates = candidates(&["ab", "cd"]);

        let config = CandidatesGridConfig::new((8, 20), &candidates, Some(2));

        // Expected printed grid (if this test passes):
        // ab  cd

        assert_eq!(config.starting_indices, vec![0, 4]);
    }

    #[test]
    fn grid_uses_default_spacing_when_none() {
        let candidates = candidates(&["a", "bbb"]);

        let config = CandidatesGridConfig::new((20, 20), &candidates, None);

        // Expected printed grid (if this test passes):
        // a  bbb

        assert_eq!(config.starting_indices, vec![0, 3]);
    }

    #[test]
    fn grid_single_candidate_wider_than_terminal_is_still_counted() {
        let candidates = candidates(&["abcdefghij"]);

        let config = CandidatesGridConfig::new((5, 20), &candidates, Some(2));

        // Expected printed grid (if this test passes):
        // abcdefghij

        assert_eq!(config.starting_indices, vec![0]);
    }

    #[test]
    fn home_case() {
        let candidates = candidates(&[
            ".git",
            ".github",
            ".gitignore",
            ".idea",
            ".pre-commit-config.yaml",
            ".vscode",
            "Cargo.lock",
            "Cargo.toml",
            "CHANGELOG.md",
            "README.md",
            "src",
            "target",
        ]);

        let config = CandidatesGridConfig::new((120, 30), &candidates, None);

        assert_eq!(
            config.starting_indices,
            vec![0, 6, 15, 27, 34, 59, 68, 80, 92, 106]
        );
    }
}
