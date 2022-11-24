use serde::Serialize;
use std::time::SystemTime;

use crate::metrics::Metrics;

#[derive(Debug, Serialize)]
pub enum HookTypes {
    #[serde(rename="ON_START")]
    OnStart,
    #[serde(rename="PING")]
    Ping,
    #[serde(rename="METRICS")]
    Metrics,
}

#[derive(Debug, Serialize)]
pub struct BaseBody {
    pub agent_id: String,
    pub sent_at: SystemTime,
    pub hook_type: HookTypes,
}

#[derive(Debug, Serialize)]
pub struct MetricsBody {
    pub agent_id: String,
    pub sent_at: SystemTime,
    pub hook_type: HookTypes,
    pub metrics: Metrics,
}
