use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use core::node::NodeRegistry;

pub async fn get(State(registry): State<NodeRegistry>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(
            registry
                .lock()
                .unwrap()
                .values()
                .map(|node_info| node_info.clone())
                .collect::<Vec<_>>(),
        ),
    )
}
