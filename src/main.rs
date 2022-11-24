#[cfg(not(any(feature = "webhooks", feature = "web")))]
compile_error!("a server feature must be enabled: 'webhooks', 'web'");
#[cfg(all(feature = "webhooks", feature = "web"))]
#[cfg(not(feature = "multi"))]
compile_error!("'multi' feature must be enabled to use multiple servers");

use agent_collector::CollectorState;
use agent_config::{readers::from_toml, types::Config};
use std::sync::Arc;
use std::time::Duration;

const CONFIG_FN: &str = "agent.toml";

#[tokio::main]
async fn main() {
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

    #[cfg(feature = "web")]
    let web_server = agent_web::run(&config, collector.clone());

    // Init Webhook if feature is enabled
    #[cfg(feature = "webhooks")]
    let webhook_server = agent_webhooks::run(&config, collector.clone());

    // Send on_start webhook and start server, if feature is enabled
    if cfg!(all(feature = "webhooks", feature = "web")) {
        // TODO switch to std::futures when it's out of experimental
        #[cfg(all(feature = "webhooks", feature = "web", feature = "multi"))]
        futures::try_join!(web_server, async {
            webhook_server.await;
            Ok(())
        })
        .unwrap();
    } else if cfg!(all(feature = "webhooks")) {
        #[cfg(feature = "webhooks")]
        webhook_server.await;
    } else if cfg!(all(feature = "web")) {
        #[cfg(feature = "web")]
        web_server.await.unwrap();
    }
}
