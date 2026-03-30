use std::{
    env::{current_dir, home_dir},
    fs::{File, read_to_string},
    io::BufWriter,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{commands::config::CommandConfig, history::HistoryConfig};

#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
pub struct PleaseConfig {
    pub command: CommandConfig,
    pub history: HistoryConfig,
}

#[derive(Serialize)]
struct PleaseConfigWithSchema<'a> {
    #[serde(flatten)]
    config: PleaseConfig,
    schema: &'a str,
}

impl Default for PleaseConfigWithSchema<'_> {
    fn default() -> Self {
        Self {
            config: Default::default(),
            schema: "https://github.com/orireiter/please-rs/releases/latest/download/please_config.schema.json",
        }
    }
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
                let config_with_schema = &PleaseConfigWithSchema::default();
                serde_json::to_writer_pretty(writer, config_with_schema).ok()
            })
            .is_none()
        {
            log::warn!("Failed to create please config in homedir")
        }

        default_conf
    }
}

// todo add test to make sure the json schema remote address actually retrieves it

#[cfg(test)]
mod tests {
    use crate::config::{PleaseConfig, PleaseConfigWithSchema};

    #[test]
    fn default_config_and_config_with_schema_are_same() {
        assert_eq!(
            PleaseConfig::default(),
            PleaseConfigWithSchema::default().config
        )
    }
}
