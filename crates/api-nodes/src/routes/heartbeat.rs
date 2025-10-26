use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use serde::Deserialize;

use gw_core::node::NodeRegistry;

#[derive(Deserialize, Debug)]
pub struct HeartbeatPayload {
    id: String,
}

pub async fn heartbeat_node(
    State(registry): State<NodeRegistry>,
    Json(payload): Json<HeartbeatPayload>,
) -> impl IntoResponse {
    let mut registry = registry.lock().unwrap();

    match registry.get_mut(&payload.id) {
        Some(node) => {
            node.last_seen = Utc::now();
            println!("[API-Nodes] Heartbeat received: id={}", payload.id);
            (StatusCode::OK, "Heartbeat received")
        }
        None => {
            println!(
                "[API-Nodes] Heartbeat of an unknown node: id={}",
                payload.id
            );
            (StatusCode::NOT_FOUND, "Unknown node")
        }
    }
}
