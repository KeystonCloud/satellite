use axum::{Router, routing::get};
use kc_core::server::ServerState;

pub mod routes;

pub fn create_router() -> Router<ServerState> {
    Router::new()
        .route("/app/{app_name}", get(routes::gateway::web_handler))
        .fallback(routes::app::fallback)
}
