use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use jsonwebtoken::{EncodingKey, Header, encode};
use sqlx::types::Uuid;

use crate::{
    models::{team::Team, user::User},
    payloads::{
        team::CreateTeamPayload,
        user::{CreateUserPayload, LoginPayload, UpdateUserPayload},
    },
};
use core::{authentication, json::DataJsonResponse, server::ServerState};

pub async fn create(
    State(state): State<ServerState>,
    Json(payload): Json<CreateUserPayload>,
) -> impl IntoResponse {
    match User::create(&state.db_pool, &payload).await {
        Ok(user) => {
            let user_team = CreateTeamPayload {
                name: format!("{}'s Team", user.name),
            };

            match Team::create(&state.db_pool, &user_team).await {
                Ok(team) => match team.associate_user(&state.db_pool, &user).await {
                    Ok(_) => (
                        StatusCode::OK,
                        Json(DataJsonResponse {
                            data: Some(user),
                            error: None,
                        }),
                    ),
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(DataJsonResponse {
                                error: Some(format!("Failed to associate user with team: {}", e)),
                                data: None,
                            }),
                        );
                    }
                },
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(DataJsonResponse {
                            error: Some(format!("Failed to create user's team: {}", e)),
                            data: None,
                        }),
                    );
                }
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DataJsonResponse {
                error: Some(format!("Failed to create user: {}", e)),
                data: None,
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

    match User::find_by_id(&state.db_pool, &uuid).await {
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

            match User::update_by_id(&state.db_pool, &uuid.to_string(), &payload).await {
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

            match User::delete_by_id(&state.db_pool, &uuid.to_string()).await {
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

pub async fn login(
    State(state): State<ServerState>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    match User::login(&state.db_pool, &payload).await {
        Ok(user) => {
            let authenticated_claims = authentication::Claims {
                user_id: user.id.to_string(),
                role: user.role,
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

pub async fn get_me(
    State(state): State<ServerState>,
    authenticated_claims: authentication::Claims,
) -> impl IntoResponse {
    match User::find_by_id(&state.db_pool, &authenticated_claims.user_id).await {
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

pub async fn update_me(
    State(state): State<ServerState>,
    authenticated_claims: authentication::Claims,
    Json(payload): Json<UpdateUserPayload>,
) -> impl IntoResponse {
    match User::update_by_id(&state.db_pool, &authenticated_claims.user_id, &payload).await {
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

pub async fn delete_me(
    State(state): State<ServerState>,
    authenticated_claims: authentication::Claims,
) -> impl IntoResponse {
    match User::delete_by_id(&state.db_pool, &authenticated_claims.user_id).await {
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
