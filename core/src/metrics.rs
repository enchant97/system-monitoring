use crate::{Bytes, Percent};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CpuLoadMetric {
    pub average: Percent,
    pub per_core: Option<Vec<Percent>>,
}

#[derive(Debug, Serialize)]
pub struct CpuMetrics {
    pub load: Option<CpuLoadMetric>,
}

#[derive(Debug, Serialize)]
pub struct MemoryDetailedMetrics {
    pub total: Bytes,
    pub available: Bytes,
    pub used: Bytes,
    pub free: Bytes,
}

#[derive(Debug, Serialize)]
pub struct MemoryMetrics {
    pub perc_used: Percent,
    pub detailed: Option<MemoryDetailedMetrics>,
}

#[derive(Debug, Serialize)]
pub struct Metrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
}
