use axum::{Json, extract::ConnectInfo, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use redis::AsyncTypedCommands;
use std::net::SocketAddr;

use core::{json::DataJsonResponse, node::NodeInfo, server::ServerState};

use crate::{models::node::Node, payloads::node::CreateNodePayload};

pub async fn post(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<ServerState>,
    Json(payload): Json<CreateNodePayload>,
) -> impl IntoResponse {
    println!("[API-Nodes] Registration received.");

    let info = NodeInfo {
        ip: addr.ip().to_string(),
        port: payload.port,
        last_seen: Utc::now().timestamp(),
    };

    match Node::create(&state.db_pool, &payload).await {
        Ok(node) => {
            println!("[API-Nodes] Node created with ID: {}", node.id);
            let info_json = match serde_json::to_string(&info) {
                Ok(json) => json,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(DataJsonResponse {
                            error: Some("JSON serialization error".to_string()),
                            data: None,
                        }),
                    );
                }
            };

            let mut conn = match state.redis_client.get_multiplexed_tokio_connection().await {
                Ok(conn) => conn,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(DataJsonResponse {
                            error: Some("Error Redis connection".to_string()),
                            data: None,
                        }),
                    );
                }
            };

            let node_key = format!("nodes:{}", node.id);
            match conn
                .set_ex::<String, String>(
                    node_key,
                    info_json,
                    state.server_settings.node_health.staleness_seconds,
                )
                .await
            {
                Ok(_) => {
                    println!("[API-Nodes] Node registered in Redis");
                    return (
                        StatusCode::OK,
                        Json(DataJsonResponse {
                            data: Some(node),
                            error: None,
                        }),
                    );
                }
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DataJsonResponse {
                        error: Some("Error in writing to Redis".to_string()),
                        data: None,
                    }),
                ),
            }
        }
        Err(e) => {
            println!("[API-Nodes] Error creating node: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DataJsonResponse {
                    error: Some("Error creating node".to_string()),
                    data: None,
                }),
            );
        }
    }
}
