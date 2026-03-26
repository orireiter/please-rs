use anyhow::{Context, Result};
use please_rs::config::PleaseConfig;
use std::{fs::File, io::BufWriter};

const JSON_SCHEMA_PATH: &str = "please_config.schema.json";
fn generate_config_schema() -> Result<()> {
    let sch = schemars::schema_for!(PleaseConfig);
    let schema_file = File::create(JSON_SCHEMA_PATH).context(format!(
        "failed to create schema file for path: {JSON_SCHEMA_PATH}"
    ))?;

    let writer = BufWriter::new(schema_file);
    serde_json::to_writer_pretty(writer, &sch)?;

    Ok(())
}

fn main() {
    if let Err(e) = generate_config_schema() {
        log::error!("Failed generating json schema for configuration, error: {e}")
    }
}
