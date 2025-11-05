use axum::{
    Router,
    routing::{delete, get, post, put},
};

use core::server::ServerState;

pub mod models;
pub mod routes;

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/", post(routes::user::create))
        .route("/", get(routes::user::get_all))
        .route("/{uuid}", get(routes::user::get))
        .route("/{uuid}", put(routes::user::update))
        .route("/{uuid}", delete(routes::user::delete))
        .with_state(state)
}
