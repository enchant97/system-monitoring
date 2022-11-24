use agent_collector::CollectorState;
use agent_config::types::{Config, WebhooksHookConfig};
use agent_core::webhooks::BaseBody;
use reqwest::Client;
use std::sync::Arc;
use std::time::SystemTime;

mod helpers;

use helpers::{new_client, sign_body};

struct WebhookManager {
    client: Client,
    config: Config,
    collector: Arc<CollectorState>,
}

impl<'a> WebhookManager {
    fn new(config: Config, collector: Arc<CollectorState>) -> Self {
        Self {
            client: new_client(),
            config,
            collector,
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
    async fn send_to_clients(&self, body: BaseBody, clients: &[WebhooksHookConfig]) {
        let raw_body = serde_json::to_string(&body).expect("unable to serialize webhook");
        // TODO switch to std::futures when it's out of experimental
        let to_send = clients
            .iter()
            .map(|hook| self.send_to_client(&raw_body, hook, &body.hook_type));
        futures::future::join_all(to_send).await;
    }
    async fn send_on_start(&self) {
        let body = BaseBody {
            agent_id: self.config.id.clone(),
            sent_at: SystemTime::now(),
            hook_type: "on_start".to_string(),
        };
        self.send_to_clients(body, &self.config.webhooks.on_start)
            .await;
    }
    async fn run(&self) {
        self.send_on_start().await;
    }
}

pub async fn run(config: &Config, collector: Arc<CollectorState>) {
    let webhook_manager = WebhookManager::new(config.clone(), collector.clone());
    webhook_manager.run().await
}
