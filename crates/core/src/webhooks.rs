use serde::Serialize;
use std::time::SystemTime;

#[derive(Debug, Serialize)]
pub struct BaseBody {
    pub agent_id: String,
    pub sent_at: SystemTime,
    pub hook_type: String,
}
