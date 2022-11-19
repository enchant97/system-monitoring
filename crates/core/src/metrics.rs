use crate::{Bytes, Percent};
use serde::Serialize;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize)]
pub struct CpuLoadMetric {
    pub average: Percent,
    pub per_core: Option<Vec<Percent>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuMetrics {
    pub load: Option<CpuLoadMetric>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryDetailedMetrics {
    pub total: Bytes,
    pub available: Bytes,
    pub used: Bytes,
    pub free: Bytes,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryMetrics {
    pub perc_used: Percent,
    pub detailed: Option<MemoryDetailedMetrics>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapturedMetrics {
    pub captured_at: SystemTime,
    pub metrics: Metrics,
}

impl CapturedMetrics {
    pub fn new_from_now(metrics: Metrics) -> Self {
        Self {
            captured_at: SystemTime::now(),
            metrics: metrics,
        }
    }
    pub fn is_old(&self, duration: Duration) -> bool {
        // TODO remove unwrap usage
        if self.captured_at.elapsed().unwrap() > duration {
            return true;
        }
        false
    }
}
