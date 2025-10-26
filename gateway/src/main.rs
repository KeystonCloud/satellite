use axum::{Router, routing::get};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use gw_core::{node::NodeRegistry, server::ServerSettings};

#[tokio::main]
async fn main() {
    let settings = ServerSettings::new().expect("Failed to load configuration");
    let server_port = settings.server.port;
    let server_host = settings.server.host;

    println!(
        "Configuration loaded. Server on {}:{}",
        server_host, server_port
    );

    let node_registry: NodeRegistry = Arc::new(Mutex::new(HashMap::new()));
    println!("State (NodeRegistry) initialized.");

    let api_nodes_router = api_nodes::create_router(node_registry.clone());

    let app = Router::new()
        .route("/", get(root_handler))
        .nest("/api/nodes", api_nodes_router);

    let addr: SocketAddr = format!("{}:{}", server_host, server_port)
        .parse()
        .expect("Invalid address format");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("Gateway started on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> &'static str {
    "Satellite online."
}
