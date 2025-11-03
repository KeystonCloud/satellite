use axum::{Router, routing::post};

use core::server::ServerState;

pub mod routes;

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/deploy", post(routes::deploy::post))
        .with_state(state)
}
