use axum::{
    Router,
    routing::{get, post},
};

use kc_core::server::ServerState;

pub mod models;
pub mod payloads;
pub mod routes;

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/mine", get(routes::app::get_mine))
        .route("/deploy", post(routes::deploy::post))
        .with_state(state)
}
