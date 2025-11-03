use axum::{Router, routing::get, routing::post};

use core::node::NodeRegistry;

pub mod routes;

pub fn create_router(state: NodeRegistry) -> Router {
    Router::new()
        .route("/list", get(routes::list::get))
        .route("/register", post(routes::register::post))
        .route("/heartbeat", post(routes::heartbeat::post))
        .with_state(state)
}
