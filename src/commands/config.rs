use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
pub struct CommandPrefixConfig {
    pub prefix_to_command_delimiter: String,
    pub prefix_elements_delimiter: String,
    pub elements: Vec<prefix_elements::PrefixElementConfig>,
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
            elements: Default::default(),
        }
    }
}

pub mod prefix_elements {
    use crossterm::style::Color;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    pub type PrefixElementConfig = (PrefixElement, ElementConfig);

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum PrefixElementDisplayParts {
        ValueOnly,
        KeyValue(String),
    }

    fn default_color() -> Color {
        Color::White
    }

    const PATTERN: &str = "^#.*|rgb_\\(\\d{1,3},\\d{1,3},\\d{1,3}\\)|Black|DarkGrey|Red|DarkRed|Green|DarkGreen|Yellow|DarkYellow|Blue|DarkBlue|Magenta|DarkMagenta|Cyan|DarkCyan|White|Grey";

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
