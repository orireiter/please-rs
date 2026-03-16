use std::{
    fs::ReadDir,
    path::{self, MAIN_SEPARATOR_STR},
};

use anyhow::Context;

use crate::{
    commands::traits::{CompletionCandidate, CompletionProvider, ConcatType},
    utils::SPACE,
};

pub fn get_completion_provider(current_command: &str) -> Box<dyn CompletionProvider> {
    // todo make the ordering configurable
    let providers: [Box<dyn CompletionProvider>; 1] = [
        // Box::new(GitCompletionProvider),
        Box::new(DirectoryCompletionProvider),
    ];

    for provider in providers {
        if provider.is_valid_provider(current_command) {
            return provider;
        }
    }

    Box::new(DirectoryCompletionProvider)
}

struct DirectoryCompletionProvider;

impl DirectoryCompletionProvider {
    fn get_current_dir_read_dir(&self) -> anyhow::Result<ReadDir> {
        let current_dir = std::env::current_dir()?;
        std::fs::read_dir(current_dir).context("failed to get read_dir for current used env")
    }
}

impl CompletionProvider for DirectoryCompletionProvider {
    fn is_valid_provider(&self, _: &str) -> bool {
        true
    }

    fn try_completing(&self, current_command: &str) -> anyhow::Result<Vec<CompletionCandidate>> {
        let mut prefix_filter = "";

        let read_dir = if let Some(last_arg) = current_command.split_whitespace().last()
            && !last_arg.is_empty()
            && !current_command.ends_with(SPACE)
        {
            let last_arg_path = path::Path::new(last_arg);

            if let Ok(dir_arg) = last_arg_path.read_dir() {
                dir_arg
            } else if let Some(parent_path) = last_arg_path.parent()
                && let Some(file_name) = last_arg_path.file_name()
                && let Some(file_name) = file_name.to_str()
            {
                prefix_filter = file_name;

                if last_arg_path.is_relative() {
                    path::Path::new(".").join(parent_path).read_dir()?
                } else {
                    parent_path.read_dir()?
                }
            } else {
                return Err(anyhow::anyhow!(
                    "failed to deconstruct elemnts of dir tab completion for last parameter"
                ));
            }
        } else if let Ok(current_dir) = self.get_current_dir_read_dir() {
            current_dir
        } else {
            return Err(anyhow::anyhow!("failed to get directory completions"));
        };

        let mut candidates = Vec::new();
        for result_entry in read_dir {
            match result_entry {
                Ok(dir_entry) => {
                    let file_name = dir_entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with(prefix_filter) {
                        let concat_type = if prefix_filter.is_empty() {
                            ConcatType::Delimited(MAIN_SEPARATOR_STR.to_string())
                        } else {
                            ConcatType::PrefixConcat(prefix_filter.len())
                        };

                        let as_candidate = CompletionCandidate::new(file_name, concat_type);
                        candidates.push(as_candidate);
                    }
                }
                Err(e) => return Err(e).context("failed to get file names in current folder"),
            }
        }

        Ok(candidates)
    }
}

#[allow(dead_code)]
struct GitCompletionProvider;

#[allow(dead_code)]
impl GitCompletionProvider {
    pub const GIT: &str = "git";
}

impl CompletionProvider for GitCompletionProvider {
    fn is_valid_provider(&self, current_command: &str) -> bool {
        current_command.trim().to_lowercase().as_str() == Self::GIT
        // todo optimize
    }

    fn try_completing(&self, _current_command: &str) -> anyhow::Result<Vec<CompletionCandidate>> {
        todo!("implement git completion")
    }
}
