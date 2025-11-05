use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use sqlx::{Row, types::Uuid};

use core::{json::SimpleJsonResponse, server::ServerState};

#[derive(Deserialize, Debug)]
pub struct CreateUserPayload {
    name: String,
    email: String,
    password: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create(
    State(state): State<ServerState>,
    Json(payload): Json<CreateUserPayload>,
) -> impl IntoResponse {
    match sqlx::query("INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING id")
        .bind(payload.name)
        .bind(payload.email)
        .bind(payload.password)
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(result) => {
            let user_id = result.get::<Uuid, _>("id");
            (
                StatusCode::OK,
                Json(SimpleJsonResponse {
                    message: format!("User created with ID: {}", user_id),
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SimpleJsonResponse {
                message: format!("Failed to create user: {}", e),
            }),
        ),
    }
}
