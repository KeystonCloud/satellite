use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub ip: String,
    pub port: u16,
    pub last_seen: DateTime<Utc>,
}

pub type NodeRegistry = Arc<Mutex<HashMap<String, NodeInfo>>>;
