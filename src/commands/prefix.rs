use crossterm::style::{ContentStyle, StyledContent};

use crate::{
    commands::{
        config::{
            CommandPrefixConfig,
            prefix_elements::{DirType, ElementConfig},
        },
        prefix::{
            element_builder::{PrefixElementBuilder, dir::DirPrefixElement},
            style::{stylize_delimiter, stylize_element_value},
        },
    },
    utils::StyledContentGroup,
};

#[derive(Default)]
pub struct LiveCommandPrefix {
    live_command_prefix_conf: CommandPrefixConfig,
    element_builders: Vec<(
        Box<dyn element_builder::PrefixElementBuilder>,
        ElementConfig,
    )>,
}

impl LiveCommandPrefix {
    pub fn new(config: Option<CommandPrefixConfig>) -> Self {
        let config = config.unwrap_or_default();
        Self {
            live_command_prefix_conf: config.clone(),
            element_builders: config
                .elements
                .iter()
                .filter_map(|(element, display)| {
                    element_builder::get_element_builder(element)
                        .map(|builder| (builder, display.clone()))
                })
                .collect(),
        }
    }

    pub fn get_command_prefix(&self) -> StyledContentGroup {
        let prefix_elements = self.build_elements();
        let elements_delimiter =
            stylize_delimiter(&self.live_command_prefix_conf.prefix_elements_delimiter);

        let mut prefix_elements_delimited = prefix_elements.join(elements_delimiter);

        let command_delimiter =
            stylize_delimiter(&self.live_command_prefix_conf.prefix_to_command_delimiter);
        prefix_elements_delimited.push(command_delimiter);
        prefix_elements_delimited
    }

    fn build_elements(&self) -> StyledContentGroup {
        if self.live_command_prefix_conf.elements.is_empty() {
            let dir_part = DirPrefixElement::new(DirType::Full).build_element();

            let stylized_dir =
                StyledContent::new(ContentStyle::default(), dir_part.unwrap_or_default());
            return StyledContentGroup::new(vec![stylized_dir]);
        }

        let mut elements = StyledContentGroup::default();
        for (builder, element_config) in &self.element_builders {
            match builder.build_element() {
                Ok(value) => {
                    let stylized_element = stylize_element_value(value.trim(), element_config);
                    elements.push(stylized_element);
                }
                Err(e) => {
                    log::warn!("failed builder prefix element, error: {e}")
                }
            }
        }
        elements
    }
}

mod element_builder {
    use anyhow::Result;

    use crate::commands::{
        config::prefix_elements::PrefixElement,
        prefix::element_builder::{
            constant_element::ConstantPrefixElement, custom::CustomPrefixElement,
            dir::DirPrefixElement, git::GitPrefixElement,
        },
    };

    pub fn get_element_builder(
        element_type: &PrefixElement,
    ) -> Option<Box<dyn PrefixElementBuilder>> {
        match element_type {
            PrefixElement::Dir(dir_type) => Some(Box::new(DirPrefixElement::new(dir_type.clone()))),
            PrefixElement::Git => Some(Box::new(GitPrefixElement::new())),
            PrefixElement::Constant(value) => {
                Some(Box::new(ConstantPrefixElement::new(value.clone())))
            }
            PrefixElement::Custom(command_conf) => {
                Some(Box::new(CustomPrefixElement::new(command_conf.clone())))
            }
        }
    }

    pub trait PrefixElementBuilder {
        fn build_element(&self) -> Result<String>;
    }
    pub mod dir {
        use std::env::current_dir;

        use anyhow::{Context, Result};

        use crate::commands::{
            config::prefix_elements::DirType, prefix::element_builder::PrefixElementBuilder,
        };

        pub struct DirPrefixElement {
            dir_type: DirType,
        }

        impl DirPrefixElement {
            pub fn new(dir_type: DirType) -> Self {
                Self { dir_type }
            }
        }

        impl PrefixElementBuilder for DirPrefixElement {
            fn build_element(&self) -> Result<String> {
                let workdir =
                    current_dir().context("failed getting current dir for prefix element")?;

                match self.dir_type {
                    DirType::Full => Ok(workdir.display().to_string()),
                    DirType::CurrentOnly => workdir
                        .file_name()
                        .map(|file_name| file_name.display().to_string())
                        .ok_or_else(|| {
                            anyhow::anyhow!("failed getting last element in current dir path")
                        }),
                    _ => todo!("implement other prefix dir types"),
                }
            }
        }
    }

    pub mod git {
        use std::{env::current_dir, fs::read_to_string, path::MAIN_SEPARATOR};

        use anyhow::Result;

        use crate::commands::prefix::element_builder::PrefixElementBuilder;

        pub struct GitPrefixElement;

        impl GitPrefixElement {
            pub fn new() -> Self {
                Self {}
            }
        }

        impl PrefixElementBuilder for GitPrefixElement {
            fn build_element(&self) -> Result<String> {
                let workdir = current_dir()?;

                for partial_part in workdir.ancestors() {
                    let partial_part_with_head =
                        partial_part.join(format!(".git{MAIN_SEPARATOR}HEAD"));

                    if let Ok(head) = read_to_string(partial_part_with_head) {
                        match head.split("/").last() {
                            Some(branch) => return Ok(branch.to_string()),
                            None => return Err(anyhow::anyhow!("malformed git HEAD file")),
                        };
                    };
                }

                Err(anyhow::anyhow!("no git repo found when traversing path"))
            }
        }
    }

    pub mod constant_element {
        use crate::commands::prefix::element_builder::PrefixElementBuilder;

        pub struct ConstantPrefixElement {
            value: String,
        }

        impl ConstantPrefixElement {
            pub fn new(value: String) -> Self {
                Self { value }
            }
        }

        impl PrefixElementBuilder for ConstantPrefixElement {
            fn build_element(&self) -> anyhow::Result<String> {
                Ok(self.value.clone())
            }
        }
    }

    pub mod custom {
        use std::process::Command;

        use anyhow::Context;

        use crate::commands::{
            config::prefix_elements::CustomPrefixElementConfig,
            prefix::element_builder::PrefixElementBuilder,
        };

        pub struct CustomPrefixElement {
            command_config: CustomPrefixElementConfig,
        }

        impl CustomPrefixElement {
            pub fn new(command_config: CustomPrefixElementConfig) -> Self {
                Self { command_config }
            }
        }

        impl PrefixElementBuilder for CustomPrefixElement {
            fn build_element(&self) -> anyhow::Result<String> {
                let result = Command::new(&self.command_config.command)
                    .args(&self.command_config.args)
                    .output()
                    .context(format!(
                        "failed to run command with config {:?}",
                        self.command_config
                    ))?;

                String::from_utf8(result.stdout).context(format!(
                    "failed to get output as string for command with config {:?}",
                    self.command_config
                ))
            }
        }
    }
}

mod style {
    use crossterm::style::{StyledContent, Stylize};

    use crate::commands::config::{
        DelimiterConfig,
        prefix_elements::{ElementConfig, PrefixElementDisplayParts},
    };

    pub fn stylize_element_value(
        value: &str,
        display_config: &ElementConfig,
    ) -> StyledContent<String> {
        let key = match &display_config.display_parts {
            PrefixElementDisplayParts::KeyValue(key) => key,
            _ => "",
        };

        let key_value_delimiter = display_config
            .key_value_delimiter
            .as_deref()
            .unwrap_or_default();

        format!("{key}{key_value_delimiter}{value}").with(display_config.color)
    }

    pub fn stylize_delimiter(delimiter_config: &DelimiterConfig) -> StyledContent<String> {
        delimiter_config
            .delimiter
            .clone()
            .with(delimiter_config.color)
    }
}

#[cfg(test)]
mod tests {
    /// tests were generated by copilot
    use std::env::current_dir;

    use crossterm::style::Color;

    use crate::commands::config::{
        CommandPrefixConfig, DelimiterConfig,
        prefix_elements::{DirType, ElementConfig, PrefixElement, PrefixElementDisplayParts},
    };

    use super::{
        LiveCommandPrefix,
        element_builder::{PrefixElementBuilder, git::GitPrefixElement},
        style::{stylize_delimiter, stylize_element_value},
    };

    #[test]
    fn default_prefix_uses_full_current_dir_and_default_command_delimiter() {
        let prefix = LiveCommandPrefix::new(None).get_command_prefix();
        let current_dir = current_dir().expect("current dir should be available");
        let expected_command_delimiter =
            stylize_delimiter(&CommandPrefixConfig::default().prefix_to_command_delimiter);

        assert_eq!(
            format!("{prefix}"),
            format!("{}{expected_command_delimiter}", current_dir.display())
        );
    }

    #[test]
    fn default_prefix_uses_only_current_dir_and_default_command_delimiter() {
        let mut conf = CommandPrefixConfig::default();
        conf.elements.push((
            PrefixElement::Dir(DirType::CurrentOnly),
            ElementConfig {
                display_parts: PrefixElementDisplayParts::ValueOnly,
                key_value_delimiter: None,
                color: Color::White,
            },
        ));
        let prefix = LiveCommandPrefix::new(Some(conf.clone())).get_command_prefix();
        let whole_cuurent_dir = current_dir().expect("current dir should be available");

        let current_dir = stylize_element_value(
            &whole_cuurent_dir
                .iter()
                .next_back()
                .expect("should have at least on folder")
                .display()
                .to_string(),
            &conf.elements[0].1,
        );
        let expected_command_delimiter = stylize_delimiter(&conf.prefix_to_command_delimiter);

        assert_eq!(
            format!("{prefix}"),
            format!("{}{expected_command_delimiter}", current_dir)
        );
    }

    #[test]
    fn prefix_respects_config_from_please_config_style() {
        let dir_element_config = ElementConfig {
            display_parts: PrefixElementDisplayParts::ValueOnly,
            key_value_delimiter: None,
            color: Color::White,
        };
        let git_element_config = ElementConfig {
            display_parts: PrefixElementDisplayParts::ValueOnly,
            key_value_delimiter: None,
            color: Color::Green,
        };

        let config = CommandPrefixConfig {
            prefix_to_command_delimiter: DelimiterConfig {
                delimiter: " -> ".to_string(),
                color: Color::White,
            },
            prefix_elements_delimiter: DelimiterConfig {
                delimiter: " | ".to_string(),
                color: Color::DarkGreen,
            },
            elements: vec![
                (
                    PrefixElement::Dir(DirType::Full),
                    dir_element_config.clone(),
                ),
                (PrefixElement::Git, git_element_config.clone()),
            ],
        };

        let prefix = LiveCommandPrefix::new(Some(config.clone())).get_command_prefix();

        let dir = current_dir()
            .expect("current dir should be available")
            .display()
            .to_string();
        let git_branch = GitPrefixElement::new()
            .build_element()
            .map(|branch| branch.trim().to_string())
            .expect("test should run inside a git repo");

        let expected = format!(
            "{}{}{}{}",
            stylize_element_value(&dir, &dir_element_config),
            stylize_delimiter(&config.prefix_elements_delimiter),
            stylize_element_value(&git_branch, &git_element_config),
            stylize_delimiter(&config.prefix_to_command_delimiter),
        );

        assert_eq!(format!("{prefix}"), expected);
    }

    #[test]
    fn key_value_display_parts_are_rendered_in_prefix_elements() {
        let dir_element_config = ElementConfig {
            display_parts: PrefixElementDisplayParts::KeyValue("cwd".to_string()),
            key_value_delimiter: Some("=".to_string()),
            color: Color::Yellow,
        };

        let config = CommandPrefixConfig {
            prefix_to_command_delimiter: DelimiterConfig {
                delimiter: " -> ".to_string(),
                color: Color::White,
            },
            prefix_elements_delimiter: DelimiterConfig {
                delimiter: " | ".to_string(),
                color: Color::DarkGreen,
            },
            elements: vec![(
                PrefixElement::Dir(DirType::Full),
                dir_element_config.clone(),
            )],
        };

        let prefix = LiveCommandPrefix::new(Some(config.clone())).get_command_prefix();
        let dir = current_dir()
            .expect("current dir should be available")
            .display()
            .to_string();

        let expected = format!(
            "{}{}",
            stylize_element_value(&dir, &dir_element_config),
            stylize_delimiter(&config.prefix_to_command_delimiter)
        );

        assert_eq!(format!("{prefix}"), expected);
    }
}
