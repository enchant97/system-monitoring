use actix_web::{get, web, web::Json};
use agent_collector::CollectorState;
use agent_config::types::Config;
use agent_core::metrics;

use crate::extractor::Client;

#[get("/is-healthy")]
pub(crate) async fn get_is_healthy() -> actix_web::Result<String> {
    Ok("🆗".to_string())
}

#[get("/agent-id")]
pub(crate) async fn get_agent_id(
    _client: Client,
    config: web::Data<Config>,
) -> actix_web::Result<String> {
    Ok(config.id.clone())
}

#[get("/")]
pub(crate) async fn get_all(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<metrics::Metrics>> {
    let captured_metrics = collector.metrics();
    Ok(Json(captured_metrics.metrics))
}

#[get("/")]
pub(crate) async fn get_cpu(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<metrics::CpuMetrics>> {
    let cpu_metrics = collector.metrics().metrics.cpu;
    Ok(Json(cpu_metrics))
}

#[get("/")]
pub(crate) async fn get_cpu_load(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<metrics::CpuLoadMetrics>> {
    let load = collector.metrics().metrics.cpu.load.unwrap();
    Ok(Json(load))
}

#[get("/average")]
pub(crate) async fn get_cpu_load_average(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<agent_core::Percent>> {
    let load = collector.metrics().metrics.cpu.load.unwrap();
    Ok(Json(load.average))
}

#[get("/per-core")]
pub(crate) async fn get_cpu_load_per_core(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<Vec<agent_core::Percent>>> {
    let load = collector.metrics().metrics.cpu.load.unwrap();
    Ok(Json(load.per_core.unwrap()))
}

#[get("/")]
pub(crate) async fn get_memory(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<metrics::MemoryMetrics>> {
    let memory_metrics = collector.metrics().metrics.memory;
    Ok(Json(memory_metrics))
}

#[get("/perc-used")]
pub(crate) async fn get_memory_perc_used(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<agent_core::Percent>> {
    let memory_metrics = collector.metrics().metrics.memory;
    Ok(Json(memory_metrics.perc_used))
}

#[get("/detailed")]
pub(crate) async fn get_memory_detailed(
    _client: Client,
    collector: web::Data<CollectorState>,
) -> actix_web::Result<Json<metrics::MemoryDetailedMetrics>> {
    let memory_metrics = collector.metrics().metrics.memory;
    Ok(Json(memory_metrics.detailed.unwrap()))
}
