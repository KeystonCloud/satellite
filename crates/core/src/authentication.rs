use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

use crate::{json::ErrorJsonResponse, server::ServerState};

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorJsonResponse {
                    error: "Authentication required".to_string(),
                }),
            ),
            AuthError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorJsonResponse {
                    error: "Invalid or expired token".to_string(),
                }),
            ),
        };
        (status, error_message).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: String,
    pub role: String,
    pub exp: usize,
}

impl FromRequestParts<ServerState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AuthError::MissingToken)?;
        let decoding_key =
            DecodingKey::from_secret(state.server_settings.server.jwt_secret.as_ref());

        let token_data = decode::<Claims>(bearer.token(), &decoding_key, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}
