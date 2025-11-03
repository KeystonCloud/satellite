use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use serde::Deserialize;

use core::{json::SimpleJsonResponse, node::NodeRegistry};

#[derive(Deserialize, Debug)]
pub struct HeartbeatPayload {
    id: String,
}

pub async fn post(
    State(registry): State<NodeRegistry>,
    Json(payload): Json<HeartbeatPayload>,
) -> impl IntoResponse {
    let mut registry = registry.lock().unwrap();

    match registry.get_mut(&payload.id) {
        Some(node) => {
            node.last_seen = Utc::now().timestamp();
            println!("[API-Nodes] Heartbeat received: id={}", payload.id);
            (
                StatusCode::OK,
                Json(SimpleJsonResponse {
                    message: format!("Heartbeat from node {} received", payload.id),
                }),
            )
        }
        None => {
            println!(
                "[API-Nodes] Heartbeat of an unknown node: id={}",
                payload.id
            );
            (
                StatusCode::NOT_FOUND,
                Json(SimpleJsonResponse {
                    message: format!("Node {} not found", payload.id),
                }),
            )
        }
    }
}
