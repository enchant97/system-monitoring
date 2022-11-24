use agent_collector::CollectorState;
use agent_config::types::{Config, WebhooksHookConfig};
use agent_core::webhooks::{BaseBody, MetricsBody};
use futures::{future::join_all, join};
use reqwest::Client;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::time::{interval, Duration};
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
        join_all(to_send).await;
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
    async fn send_interval_metrics(&self) {
        let hook_type = "metrics".to_string();
        let senders = self
            .config
            .webhooks
            .interval_metrics
            .iter()
            .map(|client| async {
                let mut interval = interval(Duration::from_secs(client.interval));
                let client_config = WebhooksHookConfig {
                    url: client.url.clone(),
                    secret: client.secret.clone(),
                };
                loop {
                    interval.tick().await;
                    let metrics = self.collector.metrics();
                    let body = MetricsBody {
                        agent_id: self.config.id.clone(),
                        sent_at: SystemTime::now(),
                        hook_type: hook_type.clone(),
                        metrics: metrics.metrics,
                    };
                    let raw_body =
                        serde_json::to_string(&body).expect("unable to serialize webhook");
                    self.send_to_client(&raw_body, &client_config, &hook_type)
                        .await;
                }
            });
        join_all(senders).await;
    }

    async fn send_interval_pings(&self) {
        let hook_type = "ping".to_string();
        let senders = self
            .config
            .webhooks
            .interval_pings
            .iter()
            .map(|client| async {
                let mut interval = interval(Duration::from_secs(client.interval));
                let client_config = WebhooksHookConfig {
                    url: client.url.clone(),
                    secret: client.secret.clone(),
                };
                loop {
                    interval.tick().await;
                    let body = BaseBody {
                        agent_id: self.config.id.clone(),
                        sent_at: SystemTime::now(),
                        hook_type: hook_type.clone(),
                    };
                    let raw_body =
                        serde_json::to_string(&body).expect("unable to serialize webhook");
                    self.send_to_client(&raw_body, &client_config, &hook_type)
                        .await;
                }
            });
        join_all(senders).await;
    }

    // run all async tasks, best used with tokio::spawn to allow aborting.
    async fn run(&self) {
        join!(
            self.send_on_start(),
            self.send_interval_pings(),
            self.send_interval_metrics()
        );
    }
}

// Start the webhook server, waiting for CTRL+C
pub async fn run(config: &Config, collector: Arc<CollectorState>) {
    let webhook_manager = WebhookManager::new(config.clone(), collector.clone());
    log::info!("starting webhooks server");
    let handle = tokio::spawn(async move { webhook_manager.run().await });
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for exit signal");
    log::info!("SIGINT received; forcing shutdown of webhooks server");
    handle.abort();
}
