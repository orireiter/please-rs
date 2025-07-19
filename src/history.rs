use std::collections::VecDeque;
use std::path::PathBuf;
use std::{cmp::min, io::Write};

use anyhow::{Context, Result};

const DEFAULT_MAX_PERSISTENT_SIZE: isize = 1_000;
const DEFAULT_PERSISTENT_FILE_NAME: &str = ".please_history";

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
}

impl History {
    pub fn from_config(config: HistoryConfig) -> Result<Self> {
        let persistent_commands = get_or_create_persistent_history_file(&config.persistent_file)?;

        Ok(Self {
            cached_history: persistent_commands,
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
