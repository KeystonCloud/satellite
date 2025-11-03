use axum::{Router, routing::get};
use core::server::ServerState;

pub mod routes;

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/app/{app_name}", get(routes::gateway::web_handler))
        .fallback(routes::app::fallback)
        .with_state(state)
}
