use actix_web::{
    dev::Payload, error::ErrorUnauthorized, get, web, web::Json, App, Error, FromRequest,
    HttpRequest, HttpServer,
};
use core::future::Future;
use monitoring_core::metrics::{CpuMetrics, MemoryMetrics, Metrics};
use psutil::cpu::CpuPercentCollector;
use std::pin::Pin;
use std::sync::Mutex;

mod config;
mod state;

use config::{read_config_or_defaults, CONFIG_FN};
use state::CollectorState;

struct Client {
    authenticated: bool,
}

impl FromRequest for Client {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Client, Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        // TODO do auth here if enabled, see below example
        // Box::pin(async { return Err(ErrorUnauthorized("")) })

        Box::pin(async {
            return Ok(Client {
                authenticated: false,
            });
        })
    }
}

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
    let collector = web::Data::new(CollectorState {
        cpu_collector: Mutex::new(CpuPercentCollector::new().unwrap()),
    });

    let config_path = std::path::PathBuf::from(CONFIG_FN);
    let config = match config_path.is_file() {
        true => read_config_or_defaults(&config_path),
        false => Default::default(),
    };

    HttpServer::new(move || {
        App::new()
            .app_data(collector.clone())
            .service(get_all)
            .service(get_cpu)
            .service(get_memory)
    })
    .bind((config.host, config.port))?
    .run()
    .await
}
