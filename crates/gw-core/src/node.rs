use chrono::Duration as ChronoDuration;
use chrono::TimeZone;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct NodeHealthConfig {
    pub staleness_seconds: u64,
    pub check_interval_seconds: u64,
}

#[derive(Serialize, Debug, Clone)]
pub struct NodeInfo {
    pub ip: String,
    pub port: u16,
    pub last_seen: i64,
}

pub type NodeRegistry = Arc<Mutex<HashMap<String, NodeInfo>>>;

pub async fn periodic_health_check(
    registry: NodeRegistry,
    check_interval_secs: u64,
    staleness_threshold_secs: u64,
) {
    let check_interval = Duration::from_secs(check_interval_secs);
    let staleness_threshold = ChronoDuration::seconds(staleness_threshold_secs as i64);

    loop {
        tokio::time::sleep(check_interval).await;

        println!("[HealthCheck] Running dead node check...");

        let mut dead_nodes_count = 0;
        let now = chrono::Utc::now();

        {
            let mut registry_lock = registry.lock().unwrap();

            registry_lock.retain(|node_id, node_info| {
                let time_since_last_seen =
                    now.signed_duration_since(Utc.timestamp_opt(node_info.last_seen, 0).unwrap());

                if time_since_last_seen > staleness_threshold {
                    println!(
                        "[HealthCheck] Dead node detected: {} (last seen: {}s)",
                        node_id,
                        time_since_last_seen.num_seconds()
                    );
                    dead_nodes_count += 1;
                    false
                } else {
                    true
                }
            });
        }

        if dead_nodes_count > 0 {
            println!(
                "[HealthCheck] Verification completed. {} node deleted.",
                dead_nodes_count
            );
        } else {
            println!("[HealthCheck] Verification completed. All nodes are active.");
        }
    }
}
