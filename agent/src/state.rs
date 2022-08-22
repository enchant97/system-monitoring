use monitoring_core::metrics::{
    CapturedMetrics, CpuLoadMetric, CpuMetrics, MemoryDetailedMetrics, MemoryMetrics, Metrics,
};
use psutil::cpu::CpuPercentCollector;
use std::sync::Mutex;
use std::time::Duration;

/// Manages gathering metrics
pub struct CollectorState {
    cache_for: Duration,
    metrics: Mutex<Option<CapturedMetrics>>,
    cpu_collector: Mutex<CpuPercentCollector>,
}

impl CollectorState {
    pub fn new(cache_for: Duration) -> Self {
        Self {
            cache_for: cache_for,
            metrics: Mutex::new(None),
            cpu_collector: Mutex::new(CpuPercentCollector::new().unwrap()),
        }
    }
    /// Gather & return cpu metrics
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
    /// Gather & return memory metrics
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

    fn metrics_refresh(&self) -> CapturedMetrics {
        return CapturedMetrics::new_from_now(Metrics {
            cpu: self.get_cpu_metrics(),
            memory: self.get_memory_metrics(),
        });
    }

    pub fn metrics(&self) -> CapturedMetrics {
        let mut stored_metrics = self.metrics.lock().unwrap();
        match &*stored_metrics {
            Some(v) => match v.is_old(self.cache_for) {
                true => {
                    log::debug!("capturing new metrics, cache old");
                    let new_metrics = self.metrics_refresh();
                    *stored_metrics = Some(new_metrics.clone());
                    new_metrics
                }
                false => {
                    log::debug!("using cached metrics");
                    v.clone()
                }
            },
            None => {
                log::debug!("capturing new metrics, updating cache");
                let new_metrics = self.metrics_refresh();
                *stored_metrics = Some(new_metrics.clone());
                new_metrics
            }
        }
    }
}
