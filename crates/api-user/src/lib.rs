use axum::{routing::post, Router};

use core::server::ServerState;

pub mod routes;

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/", post(routes::user::create))
        .with_state(state)
}
