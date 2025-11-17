use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Router,
    extract::{FromRequestParts, State},
    http::request::Parts,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use kc_core::{
    authentication::Claims,
    database::create_db_pool,
    models::query::build_schema,
    server::{ServerSettings, ServerState},
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

    let redis_client = match settings.redis.create_client() {
        Ok(client) => client,
        Err(e) => {
            panic!("Failed to create Redis client: {}", e);
        }
    };

    let graphql_schema = build_schema();

    let server_state: ServerState = ServerState {
        server_settings: settings.clone(),
        app_registry: Arc::new(Mutex::new(HashMap::new())),
        db_pool: db_pool,
        redis_client: redis_client,
        graphql_schema: graphql_schema,
    };

    let app: Router = Router::new()
        .route("/", get(root_handler))
        .nest("/api/user", api_user::create_user_router())
        .nest("/api/team", api_user::create_team_router())
        .nest("/api/node", api_node::create_router())
        .nest("/api/app", api_app::create_router())
        .route("/graphiql", get(graphiql_handler))
        .route("/graphql", post(graphql_handler))
        .merge(web_server::create_router())
        .with_state(server_state);

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

async fn graphiql_handler() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/graphql")
            .subscription_endpoint("/graphql")
            .finish(),
    )
}

async fn graphql_handler(
    State(state): State<ServerState>,
    mut parts: Parts,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let graphql_schema = state.graphql_schema.clone();
    let mut request = req.into_inner().data(state.clone());

    let claims_result = Claims::from_request_parts(&mut parts, &state).await;
    if let Ok(claims_data) = claims_result {
        request = request.data(claims_data);
    }

    graphql_schema.execute(request).await.into()
}
