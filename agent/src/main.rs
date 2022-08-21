use actix_web::{get, middleware::Logger, web, web::Json, App, HttpServer};
use monitoring_core::metrics::{CpuMetrics, MemoryMetrics, Metrics};
use psutil::cpu::CpuPercentCollector;
use std::sync::Mutex;

mod config;
mod extractor;
mod state;

use config::{read_config_toml, Config, CONFIG_FN};
use extractor::Client;
use state::CollectorState;

#[get("/")]
async fn get_all(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<Metrics>> {
    let cpu_metrics = collector.get_cpu_metrics();
    let memory_metrics = collector.get_memory_metrics();
    let metrics = Metrics {
        cpu: cpu_metrics,
        memory: memory_metrics,
    };
    Ok(Json(metrics))
}

#[get("/cpu")]
async fn get_cpu(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<CpuMetrics>> {
    let cpu_metrics = collector.get_cpu_metrics();
    Ok(Json(cpu_metrics))
}

#[get("/memory")]
async fn get_memory(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<MemoryMetrics>> {
    let memory_metrics = collector.get_memory_metrics();
    Ok(Json(memory_metrics))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    // Init metric collector
    let collector = web::Data::new(CollectorState {
        cpu_collector: Mutex::new(CpuPercentCollector::new().unwrap()),
    });
    // Load agent config
    let config_path = std::path::PathBuf::from(CONFIG_FN);
    let config: Config = match config_path.is_file() {
        true => match read_config_toml(&config_path) {
            Ok(v) => v,
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
    // Create the HTTP server
    let bind = (config.host.clone(), config.port);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(collector.clone())
            .app_data(config.clone())
            .service(get_all)
            .service(get_cpu)
            .service(get_memory)
    })
    .bind(bind)?
    .run()
    .await
}
