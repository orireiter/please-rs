use crossterm::style::Stylize;

use crate::commands::{
    config::{
        CommandPrefixConfig,
        prefix_elements::{DirType, ElementConfig, PrefixElementDisplayParts},
    },
    prefix::element_builder::{PrefixElementBuilder, dir::DirPrefixElement},
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

    pub fn get_command_prefix(&self) -> String {
        let prefix_elements = self.build_elements();
        let prefix_elements_string =
            prefix_elements.join(&self.live_command_prefix_conf.prefix_elements_delimiter);

        prefix_elements_string + &self.live_command_prefix_conf.prefix_to_command_delimiter
    }

    fn build_elements(&self) -> Vec<String> {
        if self.live_command_prefix_conf.elements.is_empty() {
            let dir_part = DirPrefixElement::new(DirType::Full).build_element();
            return vec![dir_part.unwrap_or_default()];
        }

        let mut elements = Vec::new();
        for (builder, element_config) in &self.element_builders {
            match builder.build_element() {
                Ok(value) => {
                    let stylized_element = stylize_element_value(&value, element_config);
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
        prefix::element_builder::{dir::DirPrefixElement, git::GitPrefixElement},
    };

    pub fn get_element_builder(
        element_type: &PrefixElement,
    ) -> Option<Box<dyn PrefixElementBuilder>> {
        match element_type {
            PrefixElement::Dir(dir_type) => Some(Box::new(DirPrefixElement::new(dir_type.clone()))),
            PrefixElement::Git => Some(Box::new(GitPrefixElement::new())),
            PrefixElement::Custom() => todo!(),
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
                if self.dir_type != DirType::Full {
                    todo!("implement other prefix dir types")
                }

                current_dir()
                    .map(|dir| dir.display().to_string())
                    .context("failed getting current dir for prefix element")
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
                            Some(branch) => return Ok(branch.trim().to_string()),
                            None => return Err(anyhow::anyhow!("malformed git HEAD file")),
                        };
                    };
                }

                Err(anyhow::anyhow!("no git repo found when traversing path"))
            }
        }
    }
}

fn stylize_element_value(value: &str, display_config: &ElementConfig) -> String {
    let key = match &display_config.display_parts {
        PrefixElementDisplayParts::KeyValue(key) => key,
        _ => "",
    };

    let key_value_delimiter = display_config
        .key_value_delimiter
        .as_deref()
        .unwrap_or_default();

    format!("{key}{key_value_delimiter}{value}")
        .with(display_config.color)
        .to_string()
}
