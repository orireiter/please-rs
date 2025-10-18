use anyhow::Context;

use crate::commands::traits::{CompletionCandidate, CompletionProvider, ConcatType};

pub fn get_completion_provider(_current_command: &str) -> Box<dyn CompletionProvider> {
    Box::new(DirectoryCompletionProvider)
}

struct DirectoryCompletionProvider;

impl DirectoryCompletionProvider {
    fn choose_concat_type(&self, _current_command: &str) -> ConcatType {
        /* have to choose delimiter wisely
        1. should first space from previous input?
        2. need "/" delimiter
        3. can just add adjacent and that's it?
        */
        ConcatType::Delimited("".to_string())
    }
}

impl CompletionProvider for DirectoryCompletionProvider {
    fn is_valid_provider(&self, _: &str) -> bool {
        true
    }

    fn try_completing(&self, current_command: &str) -> anyhow::Result<Vec<CompletionCandidate>> {
        if let Ok(current_dir) = std::env::current_dir()
            && let Ok(read_dir) = std::fs::read_dir(current_dir)
        {
            read_dir
                .map(|result_entry| {
                    result_entry.map(|dir_entry| {
                        let file_name = dir_entry.file_name().to_string_lossy().to_string();

                        CompletionCandidate::new(
                            file_name,
                            self.choose_concat_type(current_command),
                        )
                    })
                })
                .collect::<Result<Vec<CompletionCandidate>, _>>()
                .context("failed to get file names in current folder")
        } else {
            Err(anyhow::anyhow!("failed to get current folder"))
        }

        // todo
        // 1. support looking at last argument and trying to augment it
    }
}
