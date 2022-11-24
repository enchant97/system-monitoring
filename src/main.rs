use agent_collector::CollectorState;
use agent_config::{readers::from_toml, types::Config};
use std::sync::Arc;
use std::time::Duration;

// #[cfg(feature = "webhooks")]
// use agent_webhooks::WebhookManager;

const CONFIG_FN: &str = "agent.toml";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    if !cfg!(feature = "webhooks") {
        log::info!("built without webhooks");
    }
    // Load agent config
    let config_path = std::path::PathBuf::from(CONFIG_FN);
    let config: Config = match config_path.is_file() {
        true => match from_toml(&config_path) {
            Ok(v) => {
                log::debug!("Interpreted config file as: {v:?}");
                v
            }
            Err(_) => {
                log::warn!("config file could not be read, falling back to defaults");
                Default::default()
            }
        },
        false => {
            log::warn!("config file could not be found, falling back to defaults");
            Default::default()
        }
    };

    let collector = Arc::new(CollectorState::new(Duration::from_secs(config.cache_for)));

    let server = agent_web::run(&config, collector.clone());

    // Init Webhook if feature is enabled
    #[cfg(feature = "webhooks")]
    let webhook_server = agent_webhooks::run(&config, collector.clone());

    // Send on_start webhook and start server, if feature is enabled
    if cfg!(feature = "webhooks") {
        // TODO switch to std::futures when it's out of experimental
        #[cfg(feature = "webhooks")]
        futures::try_join!(server, async {
            webhook_server.await;
            Ok(())
        })?;
        Ok(())
    } else {
        server.await
    }
}
