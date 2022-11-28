use serde::Deserialize;
use std::net::IpAddr;
use std::path::PathBuf;
use uuid::Uuid;

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
    pub allowed_ip: Vec<IpAddr>,
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
pub struct WebConfig {
    pub host: String,
    pub port: u16,
    pub using_proxy: bool,
    pub certificate: Option<CertificateConfig>,
    pub authentication: AuthenticationConfig,
}

impl Default for WebConfig {
    fn default() -> Self {
        WebConfig {
            host: "127.0.0.1".to_string(),
            port: 9090,
            using_proxy: false,
            certificate: None,
            authentication: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhooksHookConfig {
    /// Where to send the request
    pub url: String,
    /// Used when signing the body with HMAC
    pub secret: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhooksHookConfigIntervalMetrics {
    /// Where to send the request
    pub url: String,
    /// Used when signing the body with HMAC
    pub secret: Option<String>,
    pub interval: u64,
}

impl WebhooksHookConfigIntervalMetrics {
    pub fn into_base(&self) -> WebhooksHookConfig {
        WebhooksHookConfig {
            url: self.url.clone(),
            secret: self.secret.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct WebhooksConfig {
    /// Webhook triggered when agent is starting
    pub on_start: Vec<WebhooksHookConfig>,
    pub interval_pings: Vec<WebhooksHookConfigIntervalMetrics>,
    pub interval_metrics: Vec<WebhooksHookConfigIntervalMetrics>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Agent id; used in webhooks, should be unique if using multiple agents
    pub id: String,
    pub cache_for: u64,
    pub timeout: u64,
    #[cfg(feature = "web")]
    pub web: WebConfig,
    #[cfg(feature = "webhooks")]
    pub webhooks: WebhooksConfig,
}

impl Default for Config {
    fn default() -> Self {
        let agent_uuid = Uuid::new_v4();
        Config {
            id: agent_uuid.to_string(),
            cache_for: 1,
            timeout: 4,
            #[cfg(feature = "web")]
            web: Default::default(),
            #[cfg(feature = "webhooks")]
            webhooks: Default::default(),
        }
    }
}
