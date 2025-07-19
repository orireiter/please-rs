use std::path::PathBuf;

use anyhow::Result;

const DEFAULT_MAX_PERSISTENT_SIZE: isize = 1_000;
const DEFAULT_MAX_COMMANDS_TO_PERSIST_FROM_CURRENT_SESSION: isize = 500;
const DEFAULT_PERSISTENT_FILE_NAME: &str = ".please_history";

pub struct HistoryConfig {
    persistent_file: PathBuf,
    _max_commands_in_persistent_file: isize,
    _max_commands_to_save_from_cache: isize,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        let home_dir = match std::env::home_dir() {
            Some(home_dir) => home_dir,
            None => crate::utils::HOME_DIR.into(),
        };

        Self {
            persistent_file: home_dir.join(DEFAULT_PERSISTENT_FILE_NAME),
            _max_commands_in_persistent_file: DEFAULT_MAX_PERSISTENT_SIZE,
            _max_commands_to_save_from_cache: DEFAULT_MAX_COMMANDS_TO_PERSIST_FROM_CURRENT_SESSION,
        }
    }
}

pub struct History {
    _persistent_history: Vec<String>,
    _cached_history: Vec<String>,
    _config: HistoryConfig,
}

impl History {
    pub fn from_config(config: HistoryConfig) -> Result<Self> {
        let persistent_commands = get_or_create_persistent_history_file(&config.persistent_file)?;

        Ok(Self {
            _persistent_history: persistent_commands,
            _cached_history: Vec::new(),
            _config: config,
        })
    }
}

fn get_or_create_persistent_history_file(file_path: &PathBuf) -> Result<Vec<String>> {
    let history_file_content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            if matches!(e.kind(), std::io::ErrorKind::NotFound) {
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
