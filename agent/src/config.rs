use serde::Deserialize;
use std::fs::read_to_string;
use std::path::PathBuf;

pub const CONFIG_FN: &str = "agent.toml";

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub using_proxy: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: "127.0.0.1".to_string(),
            port: 9090,
            using_proxy: false,
        }
    }
}

pub fn read_config_toml(path: &PathBuf) -> Config {
    // FIXME remove unwrap
    let raw = read_to_string(path).unwrap();
    toml::from_str(&raw).unwrap()
}

pub fn read_config_or_defaults(path: &PathBuf) -> Config {
    read_config_toml(path)
}
