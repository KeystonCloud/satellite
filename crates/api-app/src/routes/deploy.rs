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

#[derive(Deserialize, Debug, Clone)]
struct KeyInfo {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Id")]
    id: String,
}

#[derive(Deserialize, Debug)]
struct KeyListResponse {
    #[serde(rename = "Keys")]
    keys: Vec<KeyInfo>,
}

#[derive(Deserialize, Debug)]
struct IpnsPublishResponse {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Value")]
    value: String,
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

    let cid = match add_to_ipfs(&state.server_settings.server.ipfs_host, tmp_path).await {
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

    let key_info =
        match find_or_create_ipns_key(&state.server_settings.server.ipfs_host, &payload.name).await
        {
            Ok(info) => info,
            Err(e) => {
                eprintln!("[API-App] IPNS management failed: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SimpleJsonResponse {
                        message: e.to_string(),
                    }),
                );
            }
        };
    println!("[API-App] IPNS key: {}", key_info.id);

    let ipns_result = match publish_to_ipns(
        &state.server_settings.server.ipfs_host,
        &key_info.name,
        &cid,
    )
    .await
    {
        Ok(res) => res,
        Err(e) => {
            eprintln!("[API-App] IPNS publication failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SimpleJsonResponse {
                    message: e.to_string(),
                }),
            );
        }
    };
    println!(
        "[API-App] App \"{}\" published on IPNS ({} -> {})",
        payload.name, ipns_result.name, ipns_result.value
    );

    if let Err(e) = update_or_create_app_info(&state, &payload.name, &cid, &key_info) {
        eprintln!("[API-App] Create / Update app info failed: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SimpleJsonResponse {
                message: e.to_string(),
            }),
        );
    }

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

fn update_or_create_app_info(
    state: &ServerState,
    name: &str,
    cid: &str,
    key_info: &KeyInfo,
) -> Result<(), String> {
    let mut app_registry = state.app_registry.lock().unwrap();

    match app_registry.get_mut(name) {
        Some(app) => {
            app.current_cid = cid.to_string();
            app.key_name = key_info.name.clone();
            app.ipns_name = key_info.id.clone();
            println!("[API-App] Updated app info for \"{}\"", name);
        }
        None => {
            app_registry.insert(
                name.to_string(),
                AppInfo {
                    name: name.to_string(),
                    current_cid: cid.to_string(),
                    key_name: key_info.name.clone(),
                    ipns_name: key_info.id.clone(),
                },
            );
            println!("[API-App] Created new app info for \"{}\"", name);
        }
    }
    Ok(())
}

async fn find_or_create_ipns_key(ipfs_host: &str, app_name: &str) -> Result<KeyInfo, String> {
    let client = Client::new();
    let key_url = format!("{}/api/v0/key/list", ipfs_host);

    let resp = client
        .post(key_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("[IPNS] Key list error: {}", resp.status()));
    }
    let key_list: KeyListResponse = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(key) = key_list.keys.into_iter().find(|k| k.name == app_name) {
        println!("[IPNS] Key found for \"{}\"", app_name);
        return Ok(key);
    }

    println!("[IPNS] Key not found, creation for \"{}\"", app_name);
    let create_url = format!("{}/api/v0/key/gen?arg={}&type=ed25519", ipfs_host, app_name);
    let create_resp = client
        .post(create_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !create_resp.status().is_success() {
        return Err(format!("[IPNS] Key gen error: {}", create_resp.status()));
    }

    let key_info: KeyInfo = create_resp.json().await.map_err(|e| e.to_string())?;
    Ok(key_info)
}

async fn publish_to_ipns(
    ipfs_host: &str,
    key_name: &str,
    cid: &str,
) -> Result<IpnsPublishResponse, String> {
    let client = Client::new();
    let ipfs_path = format!("/ipfs/{}", cid);
    let publish_url = format!(
        "{}/api/v0/name/publish?key={}&arg={}",
        ipfs_host, key_name, ipfs_path
    );

    let resp = client
        .post(publish_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let error_text = resp.text().await.unwrap_or_default();
        return Err(format!("[IPNS] Publishing failed: {}", error_text));
    }

    let publish_info: IpnsPublishResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(publish_info)
}

async fn add_to_ipfs(ipfs_host: &str, file_path: &str) -> Result<String, String> {
    let file = fs::read(file_path).await.map_err(|e| e.to_string())?;
    let part = multipart::Part::bytes(file).file_name("deploy.tmp");
    let form = multipart::Form::new().part("file", part);

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/v0/add", ipfs_host))
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
