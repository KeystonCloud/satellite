use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Deserialize;
use sqlx::{QueryBuilder, Row, types::Uuid};
use struct_iterable::Iterable;

use crate::models::user::User;
use core::{
    authentication,
    json::{DataJsonResponse, SimpleJsonResponse},
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

pub async fn get_all(
    State(state): State<ServerState>,
    authenticated_claims: authentication::Claims,
) -> impl IntoResponse {
    if authenticated_claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(DataJsonResponse {
                data: None,
                error: Some("Insufficient permissions to access all users".to_string()),
            }),
        );
    }

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
            Json(DataJsonResponse {
                data: Some(results),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DataJsonResponse {
                data: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

async fn get_one_user_by_id(state: &ServerState, uuid: &str) -> Result<User, String> {
    match Uuid::parse_str(uuid) {
        Ok(uuid) => {
            match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(uuid)
                .fetch_one(&state.db_pool)
                .await
                .map(|mut user| {
                    user.password = None;
                    user
                }) {
                Ok(user) => Ok(user),
                Err(e) => Err(e.to_string()),
            }
        }
        Err(e) => Err(format!("Invalid UUID format: {}", e)),
    }
}

pub async fn get_me(
    State(state): State<ServerState>,
    authenticated_claims: authentication::Claims,
) -> impl IntoResponse {
    match get_one_user_by_id(&state, &authenticated_claims.user_id).await {
        Ok(user) => (
            StatusCode::OK,
            Json(DataJsonResponse {
                data: Some(user),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(DataJsonResponse {
                data: None,
                error: Some(e),
            }),
        ),
    }
}

pub async fn get(
    State(state): State<ServerState>,
    Path(uuid): Path<String>,
    authenticated_claims: authentication::Claims,
) -> impl IntoResponse {
    if !(authenticated_claims.role == "admin" || authenticated_claims.user_id == uuid) {
        return (
            StatusCode::FORBIDDEN,
            Json(DataJsonResponse {
                data: None,
                error: Some("Insufficient permissions to access this user".to_string()),
            }),
        );
    }

    match get_one_user_by_id(&state, &uuid).await {
        Ok(user) => (
            StatusCode::OK,
            Json(DataJsonResponse {
                data: Some(user),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(DataJsonResponse {
                data: None,
                error: Some(e),
            }),
        ),
    }
}

#[derive(Deserialize, Debug, Iterable)]
pub struct UpdateUserPayload {
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

pub async fn update(
    State(state): State<ServerState>,
    Path(uuid): Path<String>,
    authenticated_claims: authentication::Claims,
    Json(payload): Json<UpdateUserPayload>,
) -> impl IntoResponse {
    match Uuid::parse_str(&uuid) {
        Ok(uuid) => {
            if !(authenticated_claims.role == "admin"
                || authenticated_claims.user_id == uuid.to_string())
            {
                return (
                    StatusCode::FORBIDDEN,
                    Json(DataJsonResponse {
                        data: None,
                        error: Some("Insufficient permissions to access this user".to_string()),
                    }),
                );
            }

            let mut query_builder = QueryBuilder::new("UPDATE users");

            let mut i = 0;
            for (name, field_value) in payload.iter() {
                if let Some(value) = field_value.downcast_ref::<Option<String>>() {
                    if let Some(v) = value {
                        if i == 0 {
                            query_builder.push(" SET ");
                        } else {
                            query_builder.push(", ");
                        }

                        query_builder.push(name).push(" = ").push_bind(v);
                        i += 1;
                    }
                }
            }

            query_builder.push(" WHERE id = ").push_bind(uuid);
            query_builder.push(" RETURNING *");

            let query = query_builder.build_query_as::<User>();

            match query.fetch_one(&state.db_pool).await.map(|mut user| {
                user.password = None;
                user
            }) {
                Ok(user) => (
                    StatusCode::OK,
                    Json(DataJsonResponse {
                        data: Some(user),
                        error: None,
                    }),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DataJsonResponse {
                        data: None,
                        error: Some(e.to_string()),
                    }),
                ),
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(DataJsonResponse {
                data: None,
                error: Some(format!("Invalid UUID format: {}", e)),
            }),
        ),
    }
}

pub async fn delete(
    State(state): State<ServerState>,
    Path(uuid): Path<String>,
    authenticated_claims: authentication::Claims,
) -> impl IntoResponse {
    match Uuid::parse_str(&uuid) {
        Ok(uuid) => {
            if !(authenticated_claims.role == "admin"
                || authenticated_claims.user_id == uuid.to_string())
            {
                return (
                    StatusCode::FORBIDDEN,
                    Json(DataJsonResponse {
                        data: None,
                        error: Some("Insufficient permissions to access this user".to_string()),
                    }),
                );
            }

            match sqlx::query_as::<_, User>("DELETE FROM users WHERE id = $1 RETURNING *")
                .bind(uuid)
                .fetch_one(&state.db_pool)
                .await
            {
                Ok(user) => (
                    StatusCode::OK,
                    Json(DataJsonResponse {
                        data: Some(user),
                        error: None,
                    }),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DataJsonResponse {
                        data: None,
                        error: Some(e.to_string()),
                    }),
                ),
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(DataJsonResponse {
                data: None,
                error: Some(format!("Invalid UUID format: {}", e)),
            }),
        ),
    }
}

#[derive(Deserialize, Debug)]
pub struct LoginPayload {
    email: String,
    password: String,
}

pub async fn login(
    State(state): State<ServerState>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    match sqlx::query("SELECT * FROM users WHERE email = $1 AND password = $2")
        .bind(payload.email)
        .bind(payload.password)
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(result) => {
            let user_id = result.get::<Uuid, _>("id");
            let user_role = result.get::<String, _>("role");
            let authenticated_claims = authentication::Claims {
                user_id: user_id.to_string(),
                role: user_role,
                exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
            };
            match encode(
                &Header::default(),
                &authenticated_claims,
                &EncodingKey::from_secret(state.server_settings.server.jwt_secret.as_ref()),
            ) {
                Ok(token) => (
                    StatusCode::OK,
                    Json(DataJsonResponse {
                        data: Some(token),
                        error: None,
                    }),
                ),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DataJsonResponse {
                        error: Some(format!("Failed to generate token")),
                        data: None,
                    }),
                ),
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(DataJsonResponse {
                error: Some("User not found".to_string()),
                data: None,
            }),
        ),
    }
}
