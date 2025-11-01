use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use gw_core::node::NodeRegistry;

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
        Json(format!("App {} deployment started", payload.name)),
    )
}
