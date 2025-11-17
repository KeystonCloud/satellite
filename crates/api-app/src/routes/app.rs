use axum::{Json, extract::State, response::IntoResponse};

use kc_core::{authentication, json::DataJsonResponse, models::app::App, server::ServerState};
use reqwest::StatusCode;

pub async fn get_mine(
    State(state): State<ServerState>,
    authenticated_claims: authentication::Claims,
) -> impl IntoResponse {
    match App::find_by_user_id(&state.db_pool, &authenticated_claims.user_id).await {
        Ok(nodes) => (
            StatusCode::OK,
            Json(DataJsonResponse {
                data: Some(nodes),
                error: None,
            }),
        ),
        Err(e) => {
            println!("[API-Nodes] Nodes not found: {}", e);
            return (
                StatusCode::NOT_FOUND,
                Json(DataJsonResponse {
                    error: Some("Nodes not found".to_string()),
                    data: None,
                }),
            );
        }
    }
}
