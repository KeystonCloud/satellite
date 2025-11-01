use axum::{Router, routing::get};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use gw_core::{node::NodeRegistry, node::periodic_health_check, server::ServerSettings};

#[tokio::main]
async fn main() {
    let settings = ServerSettings::new().expect("Failed to load configuration");
    println!("Configuration loaded.");

    let node_registry: NodeRegistry = Arc::new(Mutex::new(HashMap::new()));
    println!("State (NodeRegistry) initialized.");

    let registry_clone_for_health_check = node_registry.clone();
    tokio::spawn(async move {
        println!("[HealthCheck] Background task started.");
        periodic_health_check(
            registry_clone_for_health_check,
            settings.node_health.check_interval_seconds,
            settings.node_health.staleness_seconds,
        )
        .await;
    });

    let api_node_router = api_node::create_router(node_registry.clone());
    let api_app_router = api_app::create_router(node_registry.clone());
    let app = Router::new()
        .route("/", get(root_handler))
        .nest("/api/node", api_node_router)
        .nest("/api/app", api_app_router);

    let addr: SocketAddr = format!("{}:{}", settings.server.host, settings.server.port)
        .parse()
        .expect("Invalid address format");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("Gateway started on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> &'static str {
    "Satellite online."
}
