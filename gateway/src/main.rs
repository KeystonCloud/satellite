use axum::{Router, routing::get};
use kc_core::{database::create_db_pool, server::ServerSettings, server::ServerState};
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

    let redis_client = match settings.redis.create_client() {
        Ok(client) => client,
        Err(e) => {
            panic!("Failed to create Redis client: {}", e);
        }
    };

    let server_state: ServerState = ServerState {
        server_settings: settings.clone(),
        app_registry: Arc::new(Mutex::new(HashMap::new())),
        db_pool: db_pool,
        redis_client: redis_client,
    };

    let api_user_router = api_user::create_router(server_state.clone());
    let api_node_router = api_node::create_router(server_state.clone());
    let api_app_router = api_app::create_router(server_state.clone());
    let web_server_router = web_server::create_router(server_state.clone());
    let app = Router::new()
        .route("/", get(root_handler))
        .nest("/api/user", api_user_router)
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
