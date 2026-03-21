use crate::{commands::config::CommandConfig, history::HistoryConfig};

#[derive(Clone, Default)]
pub struct PleaseConfig {
    pub command: CommandConfig,
    pub history: HistoryConfig,
}
