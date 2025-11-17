use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kc_core::{models::app::App, server::ServerState};
use reqwest::Client;

pub async fn web_handler(
    State(state): State<ServerState>,
    Path(app_name): Path<String>,
) -> Response {
    println!("[WEB] Request received for app: {}", app_name);

    let ipns_name = match App::find_by_name(&state.db_pool, &app_name).await {
        Ok(app) => match app.ipns_name {
            Some(name) => name,
            None => {
                eprintln!("[WEB] App has no IPNS name");
                return (StatusCode::NOT_FOUND, "App has no IPNS name").into_response();
            }
        },
        Err(e) => {
            eprintln!("[WEB] App not found: {}", e);
            return (StatusCode::NOT_FOUND, "App not found").into_response();
        }
    };

    println!("[WEB] App \"{}\" found. IPNS name: {}", app_name, ipns_name);

    let client = Client::new();
    let ipns_url = format!(
        "{}/api/v0/name/resolve?arg={}",
        state.server_settings.server.ipfs_host, ipns_name
    );

    let res = match client.post(&ipns_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("[WEB] IPNS error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "IPNS error").into_response();
        }
    };

    if !res.status().is_success() {
        return (StatusCode::NOT_FOUND, "Name not found in IPNS").into_response();
    }

    let cid = match res.json::<serde_json::Value>().await {
        Ok(json) => json["Path"].clone().as_str().unwrap()[6..].to_string(),
        Err(e) => {
            eprintln!("[WEB] IPNS JSON parse error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "IPNS JSON parse error").into_response();
        }
    };
    println!("[WEB] App \"{}\" found. CID: {}", app_name, cid);

    let ipfs_url = format!(
        "{}/api/v0/cat?arg={}",
        state.server_settings.server.ipfs_host, cid
    );

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
