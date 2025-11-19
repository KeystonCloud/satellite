use axum::{
    Router,
    routing::{get, post},
};

use kc_core::server::ServerState;

pub mod routes;

pub fn create_router() -> Router<ServerState> {
    Router::new()
        .route("/graphql", post(routes::graphql::handler))
        .route("/graphiql", get(routes::graphiql::handler))
}
