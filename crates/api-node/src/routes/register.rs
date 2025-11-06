use axum::{Json, extract::ConnectInfo, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use redis::AsyncTypedCommands;
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

    let info = NodeInfo {
        ip: addr.ip().to_string(),
        port: payload.port,
        last_seen: Utc::now().timestamp(),
    };

    let info_json = match serde_json::to_string(&info) {
        Ok(json) => json,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SimpleJsonResponse {
                    message: "JSON serialization error".to_string(),
                }),
            );
        }
    };

    let mut conn = match state.redis_client.get_multiplexed_tokio_connection().await {
        Ok(conn) => conn,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SimpleJsonResponse {
                    message: "Error Redis connection".to_string(),
                }),
            );
        }
    };

    let node_key = format!("nodes:{}", payload.id);
    match conn
        .set_ex::<String, String>(
            node_key,
            info_json,
            state.server_settings.node_health.staleness_seconds,
        )
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(SimpleJsonResponse {
                message: format!("Node {} registered successfully", payload.id),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SimpleJsonResponse {
                message: "Error in writing to Redis".to_string(),
            }),
        ),
    }
}
