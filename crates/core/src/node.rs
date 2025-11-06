use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct NodeHealthConfig {
    pub staleness_seconds: u64,
    pub check_interval_seconds: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeInfo {
    pub ip: String,
    pub port: u16,
    pub last_seen: i64,
}
