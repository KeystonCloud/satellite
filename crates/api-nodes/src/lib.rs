use axum::{Router, routing::post};

use gw_core::node::NodeRegistry;

pub mod routes;

pub fn create_router(state: NodeRegistry) -> Router {
    Router::new()
        .route("/register", post(routes::register::register_node))
        .route("/heartbeat", post(routes::heartbeat::heartbeat_node))
        .with_state(state)
}
