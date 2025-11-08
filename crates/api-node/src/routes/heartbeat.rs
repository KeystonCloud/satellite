use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use redis::AsyncTypedCommands;
use serde::Deserialize;

use crate::models::node::{Node, NodeInfo};
use core::{json::SimpleJsonResponse, server::ServerState};

#[derive(Deserialize, Debug)]
pub struct HeartbeatPayload {
    id: String,
}

pub async fn post(
    State(state): State<ServerState>,
    Json(payload): Json<HeartbeatPayload>,
) -> impl IntoResponse {
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

    let node = match Node::find_by_id(&state.db_pool, &payload.id).await {
        Ok(node) => node,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(SimpleJsonResponse {
                    message: format!("Node id={} not found", payload.id),
                }),
            );
        }
    };

    let node_key = format!("nodes:{}", node.id);
    let info_json: String = match conn.get(&node_key).await {
        Ok(json) => match json {
            Some(data) => data,
            None => "{\"last_seen\": null}".to_string(),
        },
        Err(_) => "{\"last_seen\": null}".to_string(),
    };

    let info_updated_json = match serde_json::from_str::<NodeInfo>(&info_json) {
        Ok(mut info) => {
            info.last_seen = Some(Utc::now().timestamp());
            serde_json::to_string(&info).unwrap_or(info_json)
        }
        Err(e) => {
            println!("Info update error: {}", e);
            info_json
        }
    };

    match conn
        .set_ex::<String, String>(
            node_key,
            info_updated_json,
            state.server_settings.node_health.staleness_seconds,
        )
        .await
    {
        Ok(_) => {
            println!("[API-Nodes] Heartbeat received: id={}", payload.id);
            (
                StatusCode::OK,
                Json(SimpleJsonResponse {
                    message: format!("Heartbeat from node {} received", payload.id),
                }),
            )
        }
        Err(_) => {
            println!("[API-Nodes] Heartbeat Redis write error");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SimpleJsonResponse {
                    message: "Redis write error".to_string(),
                }),
            )
        }
    }
}
