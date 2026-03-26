use std::{
    env::{current_dir, home_dir},
    fs::{File, read_to_string},
    io::BufWriter,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{commands::config::CommandConfig, history::HistoryConfig};

#[derive(Clone, Default, Deserialize, Serialize, JsonSchema)]
pub struct PleaseConfig {
    pub command: CommandConfig,
    pub history: HistoryConfig,
}

impl PleaseConfig {
    const CONFIG_FILENAME: &str = ".please_config";

    pub fn get_from_filesystem() -> Self {
        if let Ok(workdir_conf) =
            current_dir().and_then(|workdir| read_to_string(workdir.join(Self::CONFIG_FILENAME)))
            && !workdir_conf.is_empty()
        {
            match serde_json::from_str::<Self>(&workdir_conf) {
                Ok(conf) => return conf,
                Err(e) => {
                    log::warn!("Failed to get please config from workdir, error: {e}");
                    return Self::default();
                }
            }
        }

        let home_dir_path = home_dir().map(|path| path.join(Self::CONFIG_FILENAME));
        if let Some(homedir_conf) = home_dir_path
            .clone()
            .and_then(|homedir| read_to_string(homedir).ok())
            && !homedir_conf.is_empty()
        {
            match serde_json::from_str::<Self>(&homedir_conf) {
                Ok(conf) => return conf,
                Err(e) => {
                    log::warn!("Failed to get please config from homedir, error: {e}");
                    return Self::default();
                }
            }
        }

        let default_conf = Self::default();

        if home_dir_path
            .and_then(|p| File::create(p).ok())
            .and_then(|f| {
                let writer = BufWriter::new(f);
                serde_json::to_writer_pretty(writer, &default_conf).ok()
            })
            .is_none()
        {
            log::warn!("Failed to create please config in homedir")
        }

        default_conf
    }
}
