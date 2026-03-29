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

#[cfg(test)]
mod tests {
    use crate::commands::{
        completion::{
            dir::DirectoryCompletionProvider, get_completion_provider, git::GitCompletionProvider,
        },
        config::{CommandCompletionConfig, CommandCompletionProviderEnum},
    };

    #[test]
    fn default_completion_provider_is_dir_completion() {
        let provider = get_completion_provider("", &CommandCompletionConfig { providers: vec![] });

        let provider_string = format!("{provider:?}");

        assert_eq!(provider_string, format!("{DirectoryCompletionProvider:?}"));
    }

    #[test]
    fn git_is_selected_when_git_is_checked_before_dir_for_git_input() {
        let git_string = format!("{GitCompletionProvider:?}");
        for test_cmd in ["git", "git ", "git st"] {
            let provider = get_completion_provider(
                test_cmd,
                &CommandCompletionConfig {
                    providers: vec![
                        CommandCompletionProviderEnum::Git,
                        CommandCompletionProviderEnum::Dir,
                    ],
                },
            );

            let provider_string = format!("{provider:?}");

            assert_eq!(provider_string, git_string);
        }
    }

    #[test]
    fn dir_is_selected_when_dir_is_checked_before_git_for_git_input() {
        let provider = get_completion_provider(
            "git st",
            &CommandCompletionConfig {
                providers: vec![
                    CommandCompletionProviderEnum::Dir,
                    CommandCompletionProviderEnum::Git,
                ],
            },
        );

        let provider_string = format!("{provider:?}");

        assert_eq!(provider_string, format!("{DirectoryCompletionProvider:?}"));
    }

    #[test]
    fn dir_is_selected_for_non_git_input_even_when_git_is_checked_first() {
        let provider = get_completion_provider(
            "cargo t",
            &CommandCompletionConfig {
                providers: vec![
                    CommandCompletionProviderEnum::Git,
                    CommandCompletionProviderEnum::Dir,
                ],
            },
        );

        let provider_string = format!("{provider:?}");

        assert_eq!(provider_string, format!("{DirectoryCompletionProvider:?}"));
    }
}
