use anyhow::{Result, anyhow};
use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub log: Log,
}

#[derive(Debug, Deserialize)]
pub struct Log {
    pub filter: String,
}

#[cfg(debug_assertions)]
const SETTINGS_PATH: &str = "settings/dev.toml";
#[cfg(not(debug_assertions))]
const SETTINGS_PATH: &str = "settings/release.toml";

pub fn parse_settings(path: Option<&str>) -> Result<Settings> {
    let path = path.unwrap_or(SETTINGS_PATH);

    let settings: Settings = Config::builder()
        .add_source(File::with_name(path))
        .build()
        .map_err(|e| anyhow!(e))?
        .try_deserialize()
        .map_err(|e| anyhow!(e))?;

    Ok(settings)
}
