use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    pub prefix_config: Option<CommandPrefixConfig>,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            prefix_config: Some(CommandPrefixConfig::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPrefixConfig {
    pub prefix_to_command_delimiter: String,
    pub prefix_elements_delimiter: String,
    pub _elements: Vec<String>,
}

impl CommandPrefixConfig {
    const COMMAND_TO_PREFIX_DELIMITER: &str = " -> ";
    const PREFIX_ELEMENTS_DELIMITER: &str = "|";
}

impl Default for CommandPrefixConfig {
    fn default() -> Self {
        Self {
            prefix_to_command_delimiter: Self::COMMAND_TO_PREFIX_DELIMITER.to_string(),
            prefix_elements_delimiter: Self::PREFIX_ELEMENTS_DELIMITER.to_string(),
            _elements: Default::default(),
        }
    }
}
