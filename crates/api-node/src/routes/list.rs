use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use core::server::ServerState;

pub async fn get(State(state): State<ServerState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(
            state
                .node_registry
                .lock()
                .unwrap()
                .values()
                .map(|node_info| node_info.clone())
                .collect::<Vec<_>>(),
        ),
    )
}
