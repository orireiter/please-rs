use crossterm::style::Color;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const PATTERN: &str = "^#.*|rgb_\\(\\d{1,3},\\d{1,3},\\d{1,3}\\)|black|dark_grey|red|dark_red|green|dark_green|yellow|dark_yellow|blue|dark_blue|magenta|dark_magenta|cyan|dark_cyan|white|grey";

fn default_color() -> Color {
    Color::White
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DelimiterConfig {
    pub delimiter: String,

    #[serde(default = "default_color")]
    #[schemars(with = "String", regex(pattern = PATTERN))]
    pub color: Color,
}

impl DelimiterConfig {
    pub fn new(delimiter: Option<String>, color: Option<Color>) -> Self {
        Self {
            delimiter: delimiter.unwrap_or_default(),
            color: color.unwrap_or_else(default_color),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CommandPrefixConfig {
    pub prefix_to_command_delimiter: DelimiterConfig,
    pub prefix_elements_delimiter: DelimiterConfig,
    pub elements: Vec<prefix_elements::PrefixElementConfig>,
}

impl CommandPrefixConfig {
    const COMMAND_TO_PREFIX_DELIMITER: &str = " -> ";
    const PREFIX_ELEMENTS_DELIMITER: &str = "|";
}

impl Default for CommandPrefixConfig {
    fn default() -> Self {
        Self {
            prefix_to_command_delimiter: DelimiterConfig::new(
                Some(Self::COMMAND_TO_PREFIX_DELIMITER.to_string()),
                None,
            ),
            prefix_elements_delimiter: DelimiterConfig::new(
                Some(Self::PREFIX_ELEMENTS_DELIMITER.to_string()),
                None,
            ),
            elements: Default::default(),
        }
    }
}

pub mod prefix_elements {
    use crossterm::style::Color;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use crate::commands::config::{PATTERN, default_color};

    pub type PrefixElementConfig = (PrefixElement, ElementConfig);

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum PrefixElementDisplayParts {
        ValueOnly,
        KeyValue(String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct ElementConfig {
        pub display_parts: PrefixElementDisplayParts,
        pub key_value_delimiter: Option<String>,

        #[serde(default = "default_color")]
        #[schemars(with = "String", regex(pattern = PATTERN))]
        pub color: Color,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum PrefixElement {
        Dir(DirType),
        Git,
        Custom(),
    }

    #[derive(PartialEq, Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum DirType {
        Full,
        Shortened,
        HomeRelative,
        CurrentOnly,
    }
}
