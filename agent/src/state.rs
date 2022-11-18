use monitoring_core::metrics::{
    CapturedMetrics, CpuLoadMetric, CpuMetrics, MemoryDetailedMetrics, MemoryMetrics, Metrics,
};
use psutil::cpu::CpuPercentCollector;
use std::sync::{Mutex, RwLock};
use std::time::Duration;

/// Manages gathering metrics
pub struct CollectorState {
    cache_for: Duration,
    metrics: RwLock<Option<CapturedMetrics>>,
    cpu_collector: Mutex<CpuPercentCollector>,
}

impl CollectorState {
    pub fn new(cache_for: Duration) -> Self {
        log::debug!("Captured metrics will cache for '{cache_for:?}'");
        Self {
            cache_for: cache_for,
            metrics: RwLock::new(None),
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
    /// Return new metrics, skipping cache
    pub fn metrics_skip_cache(&self) -> CapturedMetrics {
        return CapturedMetrics::new_from_now(Metrics {
            cpu: self.get_cpu_metrics(),
            memory: self.get_memory_metrics(),
        });
    }
    /// Return metrics, using cached if valid
    pub fn metrics(&self) -> CapturedMetrics {
        let mut metrics_to_return: Option<CapturedMetrics> = None;

        // get existing metrics from cache, if they are still valid
        {
            let metrics_cache = self
                .metrics
                .read()
                .expect("cannot gain read lock on metrics cache");
            match &*metrics_cache {
                Some(v) => match v.is_old(self.cache_for) {
                    true => {
                        log::debug!("metrics capture needed, cache old");
                    }
                    false => {
                        log::debug!("metrics capture skipped, using cached");
                        metrics_to_return = Some(v.clone());
                    }
                },
                None => {
                    log::debug!("metrics capture needed, none in cache");
                }
            };
        }

        // update cached metrics
        if metrics_to_return.is_none() {
            let mut metrics_cache = self
                .metrics
                .try_write()
                .expect("cannot gain write lock on metrics cache");
            let new_metrics = self.metrics_skip_cache();
            *metrics_cache = Some(new_metrics);
            log::debug!("captured new metrics in cache");
        }

        // either used previously cached value or the newly cached metrics
        match metrics_to_return {
            Some(v) => v,
            None => self
                .metrics
                .read()
                .expect("cannot gain read lock on metrics cache")
                .as_ref()
                .expect("metrics cache cannot be None")
                .clone(),
        }
    }
}
