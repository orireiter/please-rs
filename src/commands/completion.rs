mod dir;
mod git;

use crate::commands::{
    completion::{dir::DirectoryCompletionProvider, git::GitCompletionProvider},
    traits::CompletionProvider,
};

pub fn get_completion_provider(current_command: &str) -> Box<dyn CompletionProvider> {
    // todo make the ordering configurable
    let providers: [Box<dyn CompletionProvider>; 2] = [
        Box::new(GitCompletionProvider),
        Box::new(DirectoryCompletionProvider),
    ];

    for provider in providers {
        if provider.is_valid_provider(current_command) {
            return provider;
        }
    }

    Box::new(DirectoryCompletionProvider)
}
