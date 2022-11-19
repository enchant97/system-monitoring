use crate::config::{Config, WebhooksHookConfig};
use agent_core::webhooks::BaseBody;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    redirect::Policy,
    Client,
};
use std::time::SystemTime;

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// Sign a webhooks body with HMAC-sha256
fn sign_body(body: &String, secret: String) -> String {
    let key = PKey::hmac(secret.as_bytes()).unwrap();
    let mut signer = Signer::new(MessageDigest::sha256(), &key).unwrap();
    signer.update(body.as_bytes()).unwrap();
    let signed = signer.sign_to_vec().unwrap();
    signed
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("")
}

/// Create a client ready for sending webhook requests
fn new_client() -> Client {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    Client::builder()
        .user_agent(USER_AGENT)
        .redirect(Policy::none())
        .default_headers(headers)
        .build()
        .expect("unable to build webhook client")
}

pub struct WebhookManager {
    client: Client,
    config: Config,
}

impl<'a> WebhookManager {
    pub fn new(config: Config) -> Self {
        Self {
            client: new_client(),
            config,
        }
    }
    /// Send webook to client, signing the body if required
    async fn send_to_client(
        &self,
        raw_body: &String,
        client: &WebhooksHookConfig,
        hook_type: &String,
    ) {
        let mut response = self.client.post(client.url.clone()).body(raw_body.clone());
        if client.secret.is_some() {
            // add signature header as hook has a secret
            response = response.header(
                "X-Hub-Signature-256",
                sign_body(raw_body, client.secret.clone().unwrap()),
            );
        }
        let response = response.send().await;
        match response {
            Err(err) => log::error!("failed to send webhook '{}' due to '{}'", hook_type, err),
            Ok(resp) => {
                if resp.status().is_success() {
                    log::info!(
                        "success sending webhook '{}' to '{}'",
                        hook_type,
                        client.url,
                    );
                } else {
                    log::error!(
                        "failed to send webhook '{}' to '{}' status code was '{}'",
                        hook_type,
                        client.url,
                        resp.status()
                    );
                }
            }
        };
    }
    /// Sends webhook to all clients concurrently
    async fn send_to_clients(&self, body: BaseBody, clients: &Vec<WebhooksHookConfig>) {
        let raw_body = serde_json::to_string(&body).expect("unable to serialize webhook");
        // TODO switch to std::futures when it's out of experimental
        let to_send = clients
            .iter()
            .map(|hook| self.send_to_client(&raw_body, &hook, &body.hook_type));
        futures::future::join_all(to_send).await;
    }
    pub async fn send_on_start(&self) {
        let body = BaseBody {
            agent_id: self.config.id.clone(),
            sent_at: SystemTime::now(),
            hook_type: "on_start".to_string(),
        };
        self.send_to_clients(body, &self.config.webhooks.on_start)
            .await;
    }
}
