use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use serde::Deserialize;

use gw_core::node::{NodeInfo, NodeRegistry};

#[derive(Deserialize, Debug)]
pub struct RegisterPayload {
    id: String,
    ip: String,
    port: u16,
}

pub async fn post(
    State(registry): State<NodeRegistry>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    println!("[API-Nodes] Registration received: id={}", payload.id);
    let mut registry = registry.lock().unwrap();

    let info = NodeInfo {
        ip: payload.ip,
        port: payload.port,
        last_seen: Utc::now().timestamp(),
    };
    registry.insert(payload.id.clone(), info);

    (StatusCode::OK, "Node registered")
}
