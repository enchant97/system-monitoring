use actix_web::{get, web, web::Json, App, HttpServer};
use monitoring_core::metrics::{CpuMetrics, MemoryMetrics, Metrics};
use psutil::cpu::CpuPercentCollector;
use std::sync::Mutex;

mod state;

use state::CollectorState;

#[get("/")]
async fn get_all(collector: web::Data<CollectorState>) -> actix_web::Result<Json<Metrics>> {
    let cpu_metrics = collector.get_cpu_metrics();
    let memory_metrics = collector.get_memory_metrics();
    let metrics = Metrics {
        cpu: cpu_metrics,
        memory: memory_metrics,
    };
    Ok(Json(metrics))
}

#[get("/cpu")]
async fn get_cpu(collector: web::Data<CollectorState>) -> actix_web::Result<Json<CpuMetrics>> {
    let cpu_metrics = collector.get_cpu_metrics();
    Ok(Json(cpu_metrics))
}

#[get("/memory")]
async fn get_memory(
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

    HttpServer::new(move || {
        App::new()
            .app_data(collector.clone())
            .service(get_all)
            .service(get_cpu)
            .service(get_memory)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
