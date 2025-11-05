use axum::{Json, extract::ConnectInfo, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use serde::Deserialize;
use std::net::SocketAddr;

use core::{json::SimpleJsonResponse, node::NodeInfo, server::ServerState};

#[derive(Deserialize, Debug)]
pub struct RegisterPayload {
    id: String,
    port: u16,
}

pub async fn post(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<ServerState>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    println!("[API-Nodes] Registration received: id={}", payload.id);
    let mut registry = state.node_registry.lock().unwrap();

    let info = NodeInfo {
        ip: addr.ip().to_string(),
        port: payload.port,
        last_seen: Utc::now().timestamp(),
    };
    registry.insert(payload.id.clone(), info);

    (
        StatusCode::OK,
        Json(SimpleJsonResponse {
            message: format!("Node {} registered successfully", payload.id),
        }),
    )
}
