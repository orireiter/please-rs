use anyhow::Result;

use crate::{
    commands::traits::{CompletionCandidate, CompletionProvider, ConcatType},
    utils::SPACE,
};

const GIT_COMMANDS: [&str; 44] = [
    "add",
    "am",
    "archive",
    "bisect",
    "branch",
    "bundle",
    "checkout",
    "cherry-pick",
    "citool",
    "clean",
    "clone",
    "commit",
    "describe",
    "diff",
    "fetch",
    "format-patch",
    "gc",
    "gitk",
    "grep",
    "gui",
    "init",
    "log",
    "maintenance",
    "merge",
    "mv",
    "notes",
    "pull",
    "push",
    "range-diff",
    "rebase",
    "reset",
    "restore",
    "revert",
    "rm",
    "scalar",
    "shortlog",
    "show",
    "sparse-checkout",
    "stash",
    "status",
    "submodule",
    "switch",
    "tag",
    "worktree",
];

#[derive(Debug)]
pub struct GitCompletionProvider;

impl GitCompletionProvider {
    pub const GIT: &str = "git";
}

impl CompletionProvider for GitCompletionProvider {
    fn is_valid_provider(&self, current_command: &str) -> bool {
        current_command
            .split_whitespace()
            .next()
            .map(|cmd| cmd.eq_ignore_ascii_case(Self::GIT))
            .unwrap_or(false)
    }

    fn try_completing(&self, current_command: &str) -> Result<Vec<CompletionCandidate>> {
        let mut splitted_command = current_command.split_whitespace();
        let last_element = splitted_command.next_back().unwrap_or_default();

        if splitted_command.next_back().is_none() {
            let concat_type = if current_command.trim().eq_ignore_ascii_case(Self::GIT) {
                ConcatType::Delimited(SPACE.to_string())
            } else {
                ConcatType::PrefixConcat(0)
            };

            return Ok(GIT_COMMANDS
                .iter()
                .map(|git_command| {
                    CompletionCandidate::new(git_command.to_string(), concat_type.clone())
                })
                .collect());
        }

        let filtered_commands = GIT_COMMANDS
            .iter()
            .filter_map(|git_command| match git_command.starts_with(last_element) {
                true => Some(CompletionCandidate::new(
                    git_command.to_string(),
                    crate::commands::traits::ConcatType::PrefixConcat(last_element.len()),
                )),
                false => None,
            })
            .collect();

        Ok(filtered_commands)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::commands::{
        completion::git::{GIT_COMMANDS, GitCompletionProvider},
        traits::CompletionProvider,
    };

    #[test]
    fn when_no_arg_retrieve_all_commands() {
        let all_commands: HashSet<String> = GIT_COMMANDS.iter().map(|c| c.to_string()).collect();
        for command in ["git", "git "] {
            let candidates = GitCompletionProvider {}
                .try_completing(command)
                .expect("expected to get git candidates");
            let candidate_values: Vec<String> =
                candidates.iter().map(|c| c.value.clone()).collect();

            // check they have same unique values
            assert_eq!(
                all_commands,
                HashSet::from_iter(candidate_values.iter().map(|c| c.to_string()))
            );
            // check they are the same length without dedup
            assert_eq!(all_commands.len(), candidate_values.len());
        }
    }

    #[test]
    fn filter_git_commands_by_arg_prefix() {
        for arg in ["ch", "checkout", "abc"] {
            let all_ch_commands: HashSet<String> = GIT_COMMANDS
                .iter()
                .filter_map(|c| {
                    if c.starts_with(arg) {
                        Some(c.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            let command = format!("git {arg}");
            let candidates = GitCompletionProvider {}
                .try_completing(&command)
                .expect("expected to get git candidates");
            let candidate_values: Vec<String> =
                candidates.iter().map(|c| c.value.clone()).collect();

            // check they have same unique values
            assert_eq!(
                all_ch_commands,
                HashSet::from_iter(candidate_values.iter().map(|c| c.to_string()))
            );
            // check they are the same length without dedup
            assert_eq!(all_ch_commands.len(), candidate_values.len());
        }
    }
}
