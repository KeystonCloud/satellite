use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct NodeHealthConfig {
    pub staleness_seconds: u64,
    pub check_interval_seconds: u64,
}
