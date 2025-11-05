use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::{Row, types::Uuid};

use crate::models::user::User;
use core::{
    json::{ModelJsonResponse, SimpleJsonResponse},
    server::ServerState,
};

#[derive(Deserialize, Debug)]
pub struct CreateUserPayload {
    name: String,
    email: String,
    password: String,
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

pub async fn get_all(State(state): State<ServerState>) -> impl IntoResponse {
    match sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&state.db_pool)
        .await
        .map(|results| {
            results
                .into_iter()
                .map(|mut user| {
                    user.password = None;
                    user
                })
                .collect::<Vec<User>>()
        }) {
        Ok(results) => (
            StatusCode::OK,
            Json(ModelJsonResponse {
                data: Some(results),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelJsonResponse {
                data: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

pub async fn get(State(state): State<ServerState>, Path(uuid): Path<String>) -> impl IntoResponse {
    match Uuid::parse_str(&uuid) {
        Ok(uuid) => {
            match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(uuid)
                .fetch_one(&state.db_pool)
                .await
                .map(|mut user| {
                    user.password = None;
                    user
                }) {
                Ok(user) => (
                    StatusCode::OK,
                    Json(ModelJsonResponse {
                        data: Some(user),
                        error: None,
                    }),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ModelJsonResponse {
                        data: None,
                        error: Some(e.to_string()),
                    }),
                ),
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ModelJsonResponse {
                data: None,
                error: Some(format!("Invalid UUID format: {}", e)),
            }),
        ),
    }
}
