mod dir;
mod git;

use crate::commands::{
    completion::{dir::DirectoryCompletionProvider, git::GitCompletionProvider},
    config::{CommandCompletionConfig, CommandCompletionProviderEnum},
    traits::CompletionProvider,
};

pub fn get_completion_provider(
    current_command: &str,
    completion_config: &CommandCompletionConfig,
) -> Box<dyn CompletionProvider> {
    completion_config
        .providers
        .iter()
        .find_map(|provider_enum| {
            let provider = get_provider_from_enum(provider_enum);
            if provider.is_valid_provider(current_command) {
                Some(provider)
            } else {
                None
            }
        })
        .unwrap_or_else(|| Box::new(DirectoryCompletionProvider))
}

fn get_provider_from_enum(
    provider_enum: &CommandCompletionProviderEnum,
) -> Box<dyn CompletionProvider> {
    match provider_enum {
        CommandCompletionProviderEnum::Dir => Box::new(DirectoryCompletionProvider),
        CommandCompletionProviderEnum::Git => Box::new(GitCompletionProvider),
        CommandCompletionProviderEnum::Custom => todo!(),
    }
}

// todo add tests of usage of provider config
