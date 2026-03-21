use std::{
    env::{current_dir, home_dir},
    fs::read_to_string,
};

use serde::{Deserialize, Serialize};

use crate::{commands::config::CommandConfig, history::HistoryConfig};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct PleaseConfig {
    pub command: CommandConfig,
    pub history: HistoryConfig,
}

impl PleaseConfig {
    const CONFIG_FILENAME: &str = ".please_config";

    pub fn get_from_filesystem() -> Self {
        if let Ok(workdir_conf) = current_dir().and_then(|mut workdir| {
            workdir.set_file_name(Self::CONFIG_FILENAME);
            read_to_string(workdir)
        }) && !workdir_conf.is_empty()
            && let Ok(conf) = serde_json::from_str::<Self>(&workdir_conf)
        {
            return conf;
        }

        if let Some(homedir_conf) = home_dir().and_then(|mut homedir| {
            homedir.set_file_name(Self::CONFIG_FILENAME);
            read_to_string(homedir).ok()
        }) && !homedir_conf.is_empty()
            && let Ok(conf) = serde_json::from_str::<Self>(&homedir_conf)
        {
            return conf;
        }

        // create in home and return

        Self::default()
    }
}
