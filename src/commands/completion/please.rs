use itertools::Itertools;

use crate::commands::{
    completion::try_completing_from_static_options,
    core::please::{PLEASE_COMMANDS_MAP, PleaseCommand},
    traits::CompletionProvider,
};

#[derive(Debug)]
pub struct PleaseCompletionProvider;

impl CompletionProvider for PleaseCompletionProvider {
    fn is_valid_provider(&self, current_command: &str) -> bool {
        current_command
            .split_whitespace()
            .next()
            .map(PleaseCommand::is_please_command)
            .unwrap_or(false)
    }

    fn try_completing(
        &self,
        current_command: &str,
    ) -> anyhow::Result<Vec<crate::commands::traits::CompletionCandidate>> {
        let commands: Vec<&str> = PLEASE_COMMANDS_MAP.keys().copied().sorted().collect();
        try_completing_from_static_options(
            PleaseCommand::EXECUTABLE_NAME,
            &commands,
            current_command,
        )
    }
}
