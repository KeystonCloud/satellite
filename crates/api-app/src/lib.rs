use axum::{
    Router,
    routing::{get, post},
};

use kc_core::server::ServerState;

pub mod routes;

pub fn create_router() -> Router<ServerState> {
    Router::new()
        .route("/mine", get(routes::app::get_mine))
        .route("/deploy", post(routes::deploy::post))
}
