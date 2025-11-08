use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use redis::AsyncTypedCommands;
use std::net::SocketAddr;

use crate::{
    models::node::{Node, NodeData, NodeInfo},
    payloads::node::CreateNodePayload,
};
use core::{json::DataJsonResponse, server::ServerState};

pub async fn post(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<ServerState>,
    Json(mut payload): Json<CreateNodePayload>,
) -> impl IntoResponse {
    println!("[API-Nodes] Registration received.");

    let info = NodeInfo {
        last_seen: Some(Utc::now().timestamp()),
    };
    payload.ip = Some(addr.ip().to_string());

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

pub async fn get_all(State(state): State<ServerState>) -> impl IntoResponse {
    match sqlx::query_as::<_, Node>("SELECT * FROM nodes")
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(nodes) => (
            StatusCode::OK,
            Json(DataJsonResponse {
                data: Some(nodes),
                error: None,
            }),
        ),
        Err(e) => {
            println!("[API-Nodes] Error fetching nodes from DB: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DataJsonResponse {
                    error: Some("Error fetching nodes from DB".to_string()),
                    data: None,
                }),
            );
        }
    }
}

pub async fn get(State(state): State<ServerState>, Path(uuid): Path<String>) -> impl IntoResponse {
    let node = match Node::find_by_id(&state.db_pool, &uuid).await {
        Ok(node) => node,
        Err(e) => {
            println!("[API-Nodes] Error fetching nodes from DB: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DataJsonResponse {
                    error: Some("Error fetching nodes from DB".to_string()),
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
                    error: Some("Error in Redis connection".to_string()),
                    data: None,
                }),
            );
        }
    };

    let values: Option<String> = match conn.get(format!("nodes:{}", node.id)).await {
        Ok(vals) => vals,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DataJsonResponse {
                    error: Some("Error in Redis MGET".to_string()),
                    data: None,
                }),
            );
        }
    };

    (
        StatusCode::OK,
        Json(DataJsonResponse {
            data: Some(NodeData {
                node,
                info: match values {
                    Some(info_json) => match serde_json::from_str::<NodeInfo>(&info_json) {
                        Ok(info) => Some(info),
                        Err(_) => None,
                    },
                    None => None,
                },
            }),
            error: None,
        }),
    )
}
