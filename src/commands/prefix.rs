use std::env::current_dir;

use crate::commands::config::CommandPrefixConfig;

#[derive(Default)]
pub struct LiveCommandPrefix {
    live_command_prefix_conf: CommandPrefixConfig,
}

impl LiveCommandPrefix {
    pub fn new(config: Option<CommandPrefixConfig>) -> Self {
        Self {
            live_command_prefix_conf: config.unwrap_or_default(),
        }
    }

    pub fn get_command_prefix(&self) -> String {
        let prefix_elements = self.build_elements();
        let prefix_elements_string =
            prefix_elements.join(&self.live_command_prefix_conf.prefix_elements_delimiter);

        prefix_elements_string + &self.live_command_prefix_conf.prefix_to_command_delimiter
    }

    fn build_elements(&self) -> Vec<String> {
        let dir_part = match current_dir() {
            Ok(dir) => dir.display().to_string(),
            Err(e) => format!("<error: {e}>"),
        };

        vec![dir_part]
    }
}
