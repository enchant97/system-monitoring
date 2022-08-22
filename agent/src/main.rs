use actix_web::{get, middleware::Logger, web, web::Json, App, HttpServer};
use monitoring_core::metrics;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::time::Duration;
mod config;
mod extractor;
mod state;

use config::{read_config_toml, Config, CONFIG_FN};
use extractor::Client;
use state::CollectorState;

#[get("/is-healthy")]
async fn get_is_healthy() -> actix_web::Result<String> {
    Ok("🆗".to_string())
}

#[get("/")]
async fn get_all(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<metrics::Metrics>> {
    let captured_metrics = collector.metrics();
    Ok(Json(captured_metrics.metrics))
}

#[get("/")]
async fn get_cpu(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<metrics::CpuMetrics>> {
    let cpu_metrics = collector.metrics().metrics.cpu;
    Ok(Json(cpu_metrics))
}

#[get("/")]
async fn get_cpu_load(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<metrics::CpuLoadMetric>> {
    let load = collector.metrics().metrics.cpu.load.unwrap();
    Ok(Json(load))
}

#[get("/average")]
async fn get_cpu_load_average(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<monitoring_core::Percent>> {
    let load = collector.metrics().metrics.cpu.load.unwrap();
    Ok(Json(load.average))
}

#[get("/per-core")]
async fn get_cpu_load_per_core(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<Vec<monitoring_core::Percent>>> {
    let load = collector.metrics().metrics.cpu.load.unwrap();
    Ok(Json(load.per_core.unwrap()))
}

#[get("/")]
async fn get_memory(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<metrics::MemoryMetrics>> {
    let memory_metrics = collector.metrics().metrics.memory;
    Ok(Json(memory_metrics))
}

#[get("/perc-used")]
async fn get_memory_perc_used(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<monitoring_core::Percent>> {
    let memory_metrics = collector.metrics().metrics.memory;
    Ok(Json(memory_metrics.perc_used))
}

#[get("/detailed")]
async fn get_memory_detailed(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<metrics::MemoryDetailedMetrics>> {
    let memory_metrics = collector.metrics().metrics.memory;
    Ok(Json(memory_metrics.detailed.unwrap()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    // Load agent config
    let config_path = std::path::PathBuf::from(CONFIG_FN);
    let config: Config = match config_path.is_file() {
        true => match read_config_toml(&config_path) {
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
    // Init metric collector
    let collector = web::Data::new(CollectorState::new(Duration::from_secs(config.cache_for)));
    // Create the HTTP server
    let bind = (config.host.clone(), config.port);
    let ssl_builder = match config.certificate {
        Some(ref v) => {
            // TODO remove unwrap usage
            let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
            builder
                .set_private_key_file(v.private_path.to_str().unwrap(), SslFiletype::PEM)
                .unwrap();
            builder
                .set_certificate_chain_file(v.public_path.to_str().unwrap())
                .unwrap();

            Some(builder)
        }
        None => None,
    };
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(collector.clone())
            .app_data(config.clone())
            .service(get_is_healthy)
            .service(get_all)
            .service(
                web::scope("/cpu").service(get_cpu).service(
                    web::scope("/load")
                        .service(get_cpu_load)
                        .service(get_cpu_load_average)
                        .service(get_cpu_load_per_core),
                ),
            )
            .service(
                web::scope("memory")
                    .service(get_memory)
                    .service(get_memory_perc_used)
                    .service(get_memory_detailed),
            )
    });

    match ssl_builder {
        Some(builder) => {
            log::info!("serving over HTTPS on: {bind:?}");
            server.bind_openssl(bind, builder)?.run().await
        }
        None => {
            log::info!("serving over HTTP on: {bind:?}");
            server.bind(bind)?.run().await
        }
    }
}
