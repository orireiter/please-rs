use anyhow::{Context, Result};
use please_rs::config::PleaseConfig;
use std::{fs::File, io::BufWriter};

fn generate_config_schema() -> Result<()> {
    let schema_path = "please_config.schema.json";
    let sch = schemars::schema_for!(PleaseConfig);
    let schema_file = File::create(schema_path).context(format!(
        "failed to create schema file for path: {schema_path}"
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
