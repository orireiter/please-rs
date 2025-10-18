use anyhow::Result;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ConcatType {
    /// "foo" + "bar" -> "foo{DELIMITER}bar"
    Delimited(String),
    /// "foo" + "foobar" -> "foobar"
    PrefixConcat,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct CompletionCandidate {
    pub value: String,
    pub concat_type: ConcatType,
    // add kind/type to allow style file/dir/command/argument differently
}

impl CompletionCandidate {
    #[allow(dead_code)]
    pub fn new(value: String, concat_type: ConcatType) -> Self {
        CompletionCandidate { value, concat_type }
    }
}

#[allow(dead_code)]
pub trait CompletionProvider {
    fn is_valid_provider(&self, current_command: &str) -> bool;

    fn try_completing(&self, current_command: &str) -> Result<Vec<CompletionCandidate>>;
}
