use axum::{Router, routing::post};

use gw_core::server::ServerState;

pub mod routes;

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/deploy", post(routes::deploy::post))
        .with_state(state)
}
