use axum::{
    Router,
    routing::{get, post},
};

use core::server::ServerState;

pub mod models;
pub mod routes;

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/", post(routes::user::create))
        .route("/", get(routes::user::get_all))
        .route("/{uuid}", get(routes::user::get))
        .with_state(state)
}
