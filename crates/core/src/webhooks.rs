use serde::Serialize;
use std::time::SystemTime;

use crate::metrics::Metrics;

#[derive(Debug, Serialize)]
pub struct BaseBody {
    pub agent_id: String,
    pub sent_at: SystemTime,
    pub hook_type: String,
}

#[derive(Debug, Serialize)]
pub struct MetricsBody {
    pub agent_id: String,
    pub sent_at: SystemTime,
    pub hook_type: String,
    pub metrics: Metrics,
}
