use axum::{Router, routing::post};

use kc_core::server::ServerState;

pub mod models;
pub mod payloads;
pub mod routes;

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/deploy", post(routes::deploy::post))
        .with_state(state)
}
