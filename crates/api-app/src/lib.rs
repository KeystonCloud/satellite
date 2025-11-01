use axum::{Router, routing::post};

use gw_core::node::NodeRegistry;

pub mod routes;

pub fn create_router(state: NodeRegistry) -> Router {
    Router::new()
        .route("/deploy", post(routes::deploy::post))
        .with_state(state)
}
