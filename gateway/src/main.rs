use axum::{Router, routing::get};
use core::{
    database::create_db_pool, node::periodic_health_check, server::ServerSettings,
    server::ServerState,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let settings = ServerSettings::new().expect("Failed to load configuration");

    let db_pool = match create_db_pool(&settings.database).await {
        Ok(pool) => pool,
        Err(e) => {
            panic!("Failed to create database pool: {}", e.message);
        }
    };

    let server_state: ServerState = ServerState {
        server_settings: settings.clone(),
        node_registry: Arc::new(Mutex::new(HashMap::new())),
        app_registry: Arc::new(Mutex::new(HashMap::new())),
        db_pool: db_pool,
    };

    let api_node_router = api_node::create_router(server_state.clone());
    let api_app_router = api_app::create_router(server_state.clone());
    let web_server_router = web_server::create_router(server_state.clone());
    let app = Router::new()
        .route("/", get(root_handler))
        .nest("/api/node", api_node_router)
        .nest("/api/app", api_app_router)
        .merge(web_server_router);

    let addr: SocketAddr = format!("{}:{}", settings.server.host, settings.server.port)
        .parse()
        .expect("Invalid address format");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("---- Satellite started ----");
    println!("API: {}", addr);
    println!("Gateway PEER ID: {}", settings.server.peer_id);
    println!("----------------------");

    let registry_clone_for_health_check = server_state.node_registry.clone();
    tokio::spawn(async move {
        periodic_health_check(
            registry_clone_for_health_check,
            settings.node_health.check_interval_seconds,
            settings.node_health.staleness_seconds,
        )
        .await;
    });

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn root_handler() -> &'static str {
    "Satellite online."
}
