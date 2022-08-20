use actix_web::{get, web, App, HttpServer, Responder};
use monitoring_core::metrics::{
    CpuLoadMetric, CpuMetrics, MemoryDetailedMetrics, MemoryMetrics, Metrics,
};
use psutil::cpu::CpuPercentCollector;
use std::sync::Mutex;

struct CollectorState {
    cpu_collector: Mutex<CpuPercentCollector>,
}

impl CollectorState {
    fn get_cpu_metrics(&self) -> CpuMetrics {
        let mut cpu = self.cpu_collector.lock().unwrap();

        let cpu_metrics = CpuMetrics {
            load: Some(CpuLoadMetric {
                average: cpu.cpu_percent().unwrap(),
                per_core: Some(cpu.cpu_percent_percpu().unwrap()),
            }),
        };
        cpu_metrics
    }
    fn get_memory_metrics(&self) -> MemoryMetrics {
        let memory = psutil::memory::virtual_memory().unwrap();

        let memory_metrics = MemoryMetrics {
            perc_used: memory.percent(),
            detailed: Some(MemoryDetailedMetrics {
                total: memory.total(),
                available: memory.available(),
                used: memory.used(),
                free: memory.free(),
            }),
        };
        memory_metrics
    }
}

#[get("/")]
async fn get_all(collector: web::Data<CollectorState>) -> actix_web::Result<impl Responder> {
    let cpu_metrics = collector.get_cpu_metrics();
    let memory_metrics = collector.get_memory_metrics();
    let metrics = Metrics {
        cpu: cpu_metrics,
        memory: memory_metrics,
    };
    Ok(web::Json(metrics))
}

#[get("/cpu")]
async fn get_cpu(collector: web::Data<CollectorState>) -> actix_web::Result<impl Responder> {
    let cpu_metrics = collector.get_cpu_metrics();
    Ok(web::Json(cpu_metrics))
}

#[get("/memory")]
async fn get_memory(collector: web::Data<CollectorState>) -> actix_web::Result<impl Responder> {
    let memory_metrics = collector.get_memory_metrics();
    Ok(web::Json(memory_metrics))
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
