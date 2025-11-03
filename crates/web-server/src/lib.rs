use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use core::server::ServerState;
use reqwest::Client;

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/{app_name}", get(web_handler))
        .with_state(state)
}

pub async fn web_handler(
    State(state): State<ServerState>,
    Path(app_name): Path<String>,
) -> Response {
    println!("[WEB] Request received for app: {}", app_name);

    let cid = {
        let app_registry_lock = state.app_registry.lock().unwrap();
        match app_registry_lock.get(&app_name) {
            Some(app_info) => app_info.current_cid.clone(),
            None => return (StatusCode::NOT_FOUND, "App not found").into_response(),
        }
    };

    println!("[WEB] App \"{}\" found. CID: {}", app_name, cid);

    let ipfs_url = format!(
        "{}/api/v0/cat?arg={}",
        state.server_settings.server.ipfs_host, cid
    );
    let client = Client::new();

    let res = match client.post(&ipfs_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("[WEB] IPFS error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "IPFS error").into_response();
        }
    };

    if !res.status().is_success() {
        return (StatusCode::NOT_FOUND, "CID not found in IPFS").into_response();
    }

    let body = Body::from(res.text().await.unwrap_or_default());

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(body)
        .unwrap()
}
