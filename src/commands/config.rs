use crossterm::style::Color;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const PATTERN: &str = "^(#.*|rgb_\\(\\d{1,3},\\d{1,3},\\d{1,3}\\)|black|dark_grey|red|dark_red|green|dark_green|yellow|dark_yellow|blue|dark_blue|magenta|dark_magenta|cyan|dark_cyan|white|grey)$";

fn default_color() -> Color {
    Color::White
}

#[derive(Default, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
pub struct CommandConfig {
    pub prefix_config: CommandPrefixConfig,
    pub completion_config: CommandCompletionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
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

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
    pub enum PrefixElementDisplayParts {
        ValueOnly,
        KeyValue(String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
    pub struct ElementConfig {
        pub display_parts: PrefixElementDisplayParts,
        pub key_value_delimiter: Option<String>,

        #[serde(default = "default_color")]
        #[schemars(with = "String", regex(pattern = PATTERN))]
        pub color: Color,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
    pub enum PrefixElement {
        Dir(DirType),
        Git,
        Constant(String),
        Custom(CustomPrefixElementConfig),
    }

    #[derive(PartialEq, Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub enum DirType {
        Full,
        Shortened,
        HomeRelative,
        CurrentOnly,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
    pub struct CustomPrefixElementConfig {
        pub command: String,
        pub args: Vec<String>,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CommandCompletionConfig {
    pub providers: Vec<CommandCompletionProviderEnum>,
}

impl Default for CommandCompletionConfig {
    fn default() -> Self {
        Self {
            providers: vec![
                CommandCompletionProviderEnum::Git,
                CommandCompletionProviderEnum::Dir,
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum CommandCompletionProviderEnum {
    Dir,
    Git,
    Custom,
}

#[cfg(test)]
mod tests {
    use crossterm::style::Color;
    use serde_json::json;

    use crate::commands::config::{
        CommandCompletionConfig, CommandPrefixConfig, DelimiterConfig,
        prefix_elements::{DirType, ElementConfig, PrefixElement, PrefixElementDisplayParts},
    };

    use super::CommandConfig;

    #[test]
    fn deserialize_delimiter_config_applies_default_color() {
        let cfg: DelimiterConfig =
            serde_json::from_value(json!({ "delimiter": " | " })).expect("valid delimiter");

        assert_eq!(cfg.delimiter, " | ");
        assert_eq!(cfg.color, Color::White);
    }

    #[test]
    fn deserialize_prefix_config_from_please_config_style_json() {
        let cfg: CommandPrefixConfig = serde_json::from_value(json!({
            "prefix_to_command_delimiter": { "delimiter": " -> ", "color": "#963def" },
            "prefix_elements_delimiter": { "delimiter": " | ", "color": "dark_green" },
            "elements": [
                [
                    { "Dir": "Full" },
                    { "display_parts": "ValueOnly" }
                ],
                [
                    "Git",
                    { "display_parts": "ValueOnly", "color": "green" }
                ],
            ]
        }))
        .expect("valid prefix config");

        assert_eq!(cfg.prefix_to_command_delimiter.delimiter, " -> ");
        assert_eq!(
            cfg.prefix_to_command_delimiter.color,
            Color::Rgb {
                r: 150,
                g: 61,
                b: 239
            }
        );
        assert_eq!(cfg.prefix_elements_delimiter.delimiter, " | ");
        assert_eq!(cfg.prefix_elements_delimiter.color, Color::DarkGreen);
        assert_eq!(cfg.elements.len(), 2);

        let (first_kind, first_display) = &cfg.elements[0];
        assert!(matches!(first_kind, PrefixElement::Dir(DirType::Full)));
        assert!(matches!(
            first_display.display_parts,
            PrefixElementDisplayParts::ValueOnly
        ));
        assert_eq!(first_display.key_value_delimiter, None);
        assert_eq!(first_display.color, Color::White);

        let (second_kind, second_display) = &cfg.elements[1];
        assert!(matches!(second_kind, PrefixElement::Git));
        assert!(matches!(
            second_display.display_parts,
            PrefixElementDisplayParts::ValueOnly
        ));
        assert_eq!(second_display.color, Color::Green);
    }

    #[test]
    fn serialize_command_config_emits_expected_json_shape() {
        let command_cfg = CommandConfig {
            prefix_config: CommandPrefixConfig {
                prefix_to_command_delimiter: DelimiterConfig {
                    delimiter: " => ".to_string(),
                    color: Color::Blue,
                },
                prefix_elements_delimiter: DelimiterConfig {
                    delimiter: " / ".to_string(),
                    color: Color::DarkCyan,
                },
                elements: vec![(
                    PrefixElement::Dir(DirType::CurrentOnly),
                    ElementConfig {
                        display_parts: PrefixElementDisplayParts::KeyValue("cwd".to_string()),
                        key_value_delimiter: Some("=".to_string()),
                        color: Color::Yellow,
                    },
                )],
            },
            completion_config: CommandCompletionConfig::default(),
        };

        let serialized = serde_json::to_value(command_cfg).expect("config should serialize");
        let expected = json!({
            "prefix_config": {
                "prefix_to_command_delimiter": {
                    "delimiter": " => ",
                    "color": "blue"
                },
                "prefix_elements_delimiter": {
                    "delimiter": " / ",
                    "color": "dark_cyan"
                },
                "elements": [
                    [
                        { "Dir": "CurrentOnly" },
                        {
                            "display_parts": { "KeyValue": "cwd" },
                            "key_value_delimiter": "=",
                            "color": "yellow"
                        }
                    ]
                ]
            },
            "completion_config": {
               "providers": ["Git", "Dir"]
            }
        });

        assert_eq!(serialized, expected);
    }
}
