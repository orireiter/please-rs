use std::collections::VecDeque;
use std::path::PathBuf;
use std::{cmp::min, io::Write};

use anyhow::{Context, Result};

const DEFAULT_MAX_PERSISTENT_SIZE: isize = 1_000;
const DEFAULT_PERSISTENT_FILE_NAME: &str = ".please_history";

pub enum Direction {
    Previous,
    Next,
}

pub struct HistoryConfig {
    persistent_file: PathBuf,
    max_commands_in_persistent_file: isize,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        let home_dir = match std::env::home_dir() {
            Some(home_dir) => home_dir,
            None => crate::utils::HOME_DIR.into(),
        };

        Self {
            persistent_file: home_dir.join(DEFAULT_PERSISTENT_FILE_NAME),
            max_commands_in_persistent_file: DEFAULT_MAX_PERSISTENT_SIZE,
        }
    }
}

pub struct History {
    cached_history: VecDeque<String>,
    config: HistoryConfig,
    last_stopped_index: Option<usize>,
}

impl History {
    pub fn from_config(config: HistoryConfig) -> Result<Self> {
        let persistent_commands = get_or_create_persistent_history_file(&config.persistent_file)?;

        Ok(Self {
            cached_history: persistent_commands,
            last_stopped_index: None,
            config,
        })
    }

    pub fn add_command_to_cache(&mut self, command: String) {
        self.cached_history.push_front(command);
    }

    pub fn save_history_to_persistent_file(&self) -> Result<()> {
        let mut file = std::fs::File::create(&self.config.persistent_file)?;

        let mut command_count = self.cached_history.len();
        if self.config.max_commands_in_persistent_file.is_positive() {
            command_count = min(
                command_count,
                self.config.max_commands_in_persistent_file as usize,
            );
        }

        for index in 0..command_count {
            let command = &self.cached_history[index];
            writeln!(file, "{command}").context(format!(
                "failed to save \"{command}\" to {}",
                self.config.persistent_file.to_string_lossy()
            ))?;
        }

        Ok(())
    }

    pub fn navigate_to_previous(&mut self, pattern: &str) -> Option<&str> {
        if let Some(last_stopped_index) = self.last_stopped_index
            && last_stopped_index >= self.cached_history.len()
        {
            return None;
        }

        let history_slice = self
            .cached_history
            .range(self.last_stopped_index.unwrap_or_default()..);

        let current_historical_command = if let Some(last_stopped_index) = self.last_stopped_index {
            &self.cached_history[last_stopped_index]
        } else {
            ""
        };

        for (index, historical_command) in history_slice.enumerate() {
            if historical_command.starts_with(pattern)
                && current_historical_command.ne(historical_command)
            {
                let new_index = index + self.last_stopped_index.unwrap_or_default();

                self.last_stopped_index = Some(self.cached_history.len().min(new_index));
                return Some(historical_command);
            }
        }

        self.last_stopped_index = Some(self.cached_history.len().saturating_sub(1));
        None
    }

    pub fn navigate_to_next(&mut self, pattern: &str) -> Option<&str> {
        let last_stopped_index = self.last_stopped_index?;

        if last_stopped_index == 0 {
            return None;
        }

        let history_slice = self.cached_history.range(..last_stopped_index);

        let current_historical_command = &self.cached_history[last_stopped_index];
        for (index, historical_command) in history_slice.rev().enumerate() {
            if historical_command.starts_with(pattern)
                && current_historical_command.ne(historical_command)
            {
                self.last_stopped_index = Some(0.max(last_stopped_index - index - 1));
                return Some(historical_command);
            }
        }

        self.last_stopped_index = None;
        None
    }

    pub fn reset_history_search_index(&mut self) {
        self.last_stopped_index = None
    }
}

fn get_or_create_persistent_history_file(file_path: &PathBuf) -> Result<VecDeque<String>> {
    let history_file_content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            if matches!(e.kind(), std::io::ErrorKind::NotFound) {
                log::debug!(
                    "creating please history file in {}",
                    file_path.to_string_lossy()
                );
                std::fs::File::create(file_path)?;
            }

            String::new()
        }
    };

    Ok(history_file_content
        .split_terminator(crate::utils::NEWLINE)
        .map(|command_string| command_string.to_string())
        .collect())
}
