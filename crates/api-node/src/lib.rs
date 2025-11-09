use axum::{Router, routing::get, routing::post};

use kc_core::server::ServerState;

pub mod models;
pub mod payloads;
pub mod routes;

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/", get(routes::node::get_all))
        .route("/{uuid}", get(routes::node::get))
        .route("/mine", get(routes::node::get_mine))
        .route("/", post(routes::node::post))
        .route("/heartbeat", post(routes::heartbeat::post))
        .with_state(state)
}
