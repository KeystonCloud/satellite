use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use gw_core::{json::SimpleJsonResponse, node::NodeRegistry};

#[derive(Deserialize, Debug)]
pub struct AppDeployPayload {
    name: String,
}

pub async fn post(
    State(_registry): State<NodeRegistry>,
    Json(payload): Json<AppDeployPayload>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(SimpleJsonResponse {
            message: format!("Deploy request for app {} received", payload.name),
        }),
    )
}
