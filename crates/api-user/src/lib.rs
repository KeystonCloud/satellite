use axum::{
    Router,
    routing::{delete, get, post, put},
};

use kc_core::server::ServerState;

pub mod models;
pub mod payloads;
pub mod routes;
pub mod utils;

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/", post(routes::user::create))
        .route("/", get(routes::user::get_all))
        .route("/{uuid}", get(routes::user::get))
        .route("/{uuid}", put(routes::user::update))
        .route("/{uuid}", delete(routes::user::delete))
        .route("/login", post(routes::user::login))
        .route("/me", get(routes::user::get_me))
        .route("/me", put(routes::user::update_me))
        .route("/me", delete(routes::user::delete_me))
        .with_state(state)
}
