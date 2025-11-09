use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::types::Uuid;

use crate::{
    models::team::Team,
    payloads::team::{CreateTeamPayload, UpdateTeamPayload},
};
use kc_core::{authentication, json::DataJsonResponse, server::ServerState};

pub async fn create(
    State(state): State<ServerState>,
    authenticated_claims: authentication::Claims,
    Json(payload): Json<CreateTeamPayload>,
) -> impl IntoResponse {
    match Team::create(&state.db_pool, &payload).await {
        Ok(team) => {
            match team
                .associate_user_by_id(&state.db_pool, &authenticated_claims.user_id)
                .await
            {
                Ok(_) => (
                    StatusCode::CREATED,
                    Json(DataJsonResponse {
                        error: None,
                        data: Some(team),
                    }),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DataJsonResponse {
                        error: Some(format!("Failed to associate user with team: {}", e)),
                        data: None,
                    }),
                ),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DataJsonResponse {
                error: Some(format!("Failed to create team: {}", e)),
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
                error: Some("Insufficient permissions to access all teams".to_string()),
            }),
        );
    }

    match sqlx::query_as::<_, Team>("SELECT * FROM teams")
        .fetch_all(&state.db_pool)
        .await
        .map(|results| results.into_iter().collect::<Vec<Team>>())
    {
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
                error: Some("Insufficient permissions to access this team".to_string()),
            }),
        );
    }

    match Team::find_by_id(&state.db_pool, &uuid).await {
        Ok(team) => (
            StatusCode::OK,
            Json(DataJsonResponse {
                data: Some(team),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
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
    Json(payload): Json<UpdateTeamPayload>,
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
                        error: Some("Insufficient permissions to access this team".to_string()),
                    }),
                );
            }

            match Team::update_by_id(&state.db_pool, &uuid.to_string(), &payload).await {
                Ok(team) => (
                    StatusCode::OK,
                    Json(DataJsonResponse {
                        data: Some(team),
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
                        error: Some("Insufficient permissions to access this team".to_string()),
                    }),
                );
            }

            match Team::delete_by_id(&state.db_pool, &uuid.to_string()).await {
                Ok(team) => (
                    StatusCode::OK,
                    Json(DataJsonResponse {
                        data: Some(team),
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

pub async fn get_mine(
    State(state): State<ServerState>,
    authenticated_claims: authentication::Claims,
) -> impl IntoResponse {
    match Team::find_by_user_id(&state.db_pool, &authenticated_claims.user_id).await {
        Ok(results) => (
            StatusCode::OK,
            Json(DataJsonResponse {
                data: Some(results),
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
