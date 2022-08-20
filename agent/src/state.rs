use monitoring_core::metrics::{CpuLoadMetric, CpuMetrics, MemoryDetailedMetrics, MemoryMetrics};
use psutil::cpu::CpuPercentCollector;
use std::sync::Mutex;

/// Manages gathering metrics
pub struct CollectorState {
    pub cpu_collector: Mutex<CpuPercentCollector>,
}

impl CollectorState {
    /// Gather & return cpu metrics
    pub fn get_cpu_metrics(&self) -> CpuMetrics {
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
    pub fn get_memory_metrics(&self) -> MemoryMetrics {
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
