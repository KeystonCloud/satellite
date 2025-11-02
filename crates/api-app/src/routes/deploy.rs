use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use tokio::fs;

use gw_core::{app::AppInfo, json::SimpleJsonResponse, server::ServerState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppDeployPayload {
    name: String,
    content: String,
}

#[derive(Serialize, Debug)]
pub struct NodeDeployPayload {
    name: String,
    cid: String,
}

#[derive(Deserialize, Debug)]
struct IPFSAddResponse {
    #[serde(rename = "Hash")]
    hash: String,
}

pub async fn post(
    State(state): State<ServerState>,
    Json(payload): Json<AppDeployPayload>,
) -> impl IntoResponse {
    let tmp_path = "/tmp/keystone_deploy.tmp";
    if let Err(e) = fs::write(tmp_path, payload.content).await {
        eprintln!("[API-App] Error in file creation: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SimpleJsonResponse {
                message: "Error in file creation".to_string(),
            }),
        );
    }

    let cid = match add_to_ipfs(tmp_path).await {
        Ok(cid) => cid,
        Err(e) => {
            eprintln!("[API-App] Add to IPFS failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SimpleJsonResponse {
                    message: e.to_string(),
                }),
            );
        }
    };
    println!("[API-App] File added to IPFS. CID: {}", cid);

    let app_info = AppInfo {
        name: payload.name.clone(),
        current_cid: cid.clone(),
    };
    state
        .app_registry
        .lock()
        .unwrap()
        .insert(payload.name.clone(), app_info);

    let client = Client::new();
    let nodes_to_deploy = state.node_registry.lock().unwrap().clone(); // TODO: filter nodes based on criteria (geo, capacity, reputation...)

    // EXAMPLE : send deployment on all nodes
    for (id, node) in nodes_to_deploy {
        let client_clone = client.clone();
        let deploy_url = format!("http://{}:{}/api/deploy", node.ip, node.port);
        let node_payload = NodeDeployPayload {
            name: payload.name.clone(),
            cid: cid.clone(),
        };

        tokio::spawn(async move {
            let res = client_clone
                .post(&deploy_url)
                .json(&node_payload)
                .send()
                .await;

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("[API-App] Send app deployment to node: id={}", id);
                    } else {
                        println!(
                            "[API-App] Failed to app deployment to node: id={}, status={}",
                            id,
                            response.status()
                        );
                    }
                }
                Err(e) => {
                    println!(
                        "[API-App] Error sending app deployment to node: id={}, error={}",
                        id, e
                    );
                }
            }
        });
    }

    (
        StatusCode::OK,
        Json(SimpleJsonResponse {
            message: format!(
                "App \"{}\" deployment initiated with CID: {}",
                payload.name, cid
            ),
        }),
    )
}

async fn add_to_ipfs(file_path: &str) -> Result<String, String> {
    let file = fs::read(file_path).await.map_err(|e| e.to_string())?;
    let part = multipart::Part::bytes(file).file_name("deploy.tmp");
    let form = multipart::Form::new().part("file", part);

    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:5001/api/v0/add")
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("[API-App] IPFS error {}", resp.status()));
    }

    let kubo_resp: IPFSAddResponse = resp.json().await.map_err(|e| e.to_string())?;

    Ok(kubo_resp.hash)
}
