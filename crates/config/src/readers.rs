use crate::errors::ConfigError;
use crate::types::Config;
use std::fs::read_to_string;
use std::path::PathBuf;

/// Read the agent config from a TOML file
pub fn from_toml(path: &PathBuf) -> Result<Config, ConfigError> {
    match read_to_string(path) {
        Ok(raw) => match toml::from_str(&raw) {
            Ok(config) => Ok(config),
            Err(_) => Err(ConfigError::ParseError),
        },
        Err(_) => Err(ConfigError::ReadError),
    }
}
