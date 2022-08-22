use serde::Deserialize;
use std::fs::read_to_string;
use std::path::PathBuf;

pub const CONFIG_FN: &str = "agent.toml";

pub enum ConfigError {
    ReadError,
    ParseError,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CertificateConfig {
    pub private_path: PathBuf,
    pub public_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuthenticationConfig {
    pub check_ip: bool,
    pub check_key: bool,
    pub allowed_ip: Vec<String>,
    pub allowed_keys: Vec<String>,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        AuthenticationConfig {
            check_ip: false,
            check_key: false,
            allowed_ip: vec![],
            allowed_keys: vec![],
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub using_proxy: bool,
    pub cache_for: u64,
    pub certificate: Option<CertificateConfig>,
    pub authentication: AuthenticationConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: "127.0.0.1".to_string(),
            port: 9090,
            using_proxy: false,
            cache_for: 1,
            certificate: None,
            authentication: Default::default(),
        }
    }
}

/// Read the agent config from a TOML file
pub fn read_config_toml(path: &PathBuf) -> Result<Config, ConfigError> {
    match read_to_string(path) {
        Ok(raw) => match toml::from_str(&raw) {
            Ok(config) => Ok(config),
            Err(_) => Err(ConfigError::ParseError),
        },
        Err(_) => Err(ConfigError::ReadError),
    }
}
