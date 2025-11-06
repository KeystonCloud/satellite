use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use redis::{AsyncIter, AsyncTypedCommands};

use core::{json::DataJsonResponse, node::NodeInfo, server::ServerState};

pub async fn get(State(state): State<ServerState>) -> impl IntoResponse {
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

    let keys: Vec<String> = {
        let mut iter: AsyncIter<String> = match conn.scan_match("nodes:*").await {
            Ok(iter) => iter,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DataJsonResponse {
                        error: Some("Error in Redis SCAN".to_string()),
                        data: None,
                    }),
                );
            }
        };

        let mut keys_tmp = Vec::new();
        while let Some(key) = iter.next_item().await {
            match key {
                Ok(k) => keys_tmp.push(k),
                Err(_) => (),
            }
        }

        keys_tmp
    };

    if keys.is_empty() {
        return (
            StatusCode::OK,
            Json(DataJsonResponse {
                data: Some(Vec::<NodeInfo>::new()),
                error: None,
            }),
        );
    }

    let values: Vec<Option<String>> = match conn.mget(keys).await {
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

    let nodes: Vec<NodeInfo> = values
        .into_iter()
        .filter_map(|json_str| serde_json::from_str(&json_str.unwrap()).ok())
        .collect();

    (
        StatusCode::OK,
        Json(DataJsonResponse {
            data: Some(nodes),
            error: None,
        }),
    )
}
