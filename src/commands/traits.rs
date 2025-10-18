use anyhow::Result;

#[derive(Debug)]
pub enum ConcatType {
    /// "foo" + "bar" -> "foo{DELIMITER}bar"
    Delimited(String),

    /// "foo" + "foobar" -> "foobar"
    #[allow(dead_code)]
    PrefixConcat,
}

#[derive(Debug)]
pub struct CompletionCandidate {
    pub value: String,
    pub concat_type: ConcatType,
    // add kind/type to allow style file/dir/command/argument differently
}

impl CompletionCandidate {
    pub fn new(value: String, concat_type: ConcatType) -> Self {
        CompletionCandidate { value, concat_type }
    }
}

pub trait CompletionProvider {
    #[allow(dead_code)]
    fn is_valid_provider(&self, current_command: &str) -> bool;

    fn try_completing(&self, current_command: &str) -> Result<Vec<CompletionCandidate>>;
}
