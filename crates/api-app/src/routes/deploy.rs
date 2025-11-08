use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use tokio::fs;

use api_node::models::node::Node;
use core::{database::DbPool, json::DataJsonResponse, server::ServerState};
use std::collections::HashMap;

use crate::{
    models::{
        app::App,
        deployment::{Deployment, DeploymentStatus},
        deployment_node::{DeploymentNode, PinStatus},
    },
    payloads::{
        app::{CreateAppPayload, UpdateAppPayload},
        deployment::{CreateDeploymentPayload, UpdateDeploymentPayload},
        deployment_node::{CreateDeploymentNodePayload, UpdateDeploymentNodePayload},
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppDeployPayload {
    id: Option<String>,
    team_id: Option<String>,
    name: Option<String>,
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
    // Temporary store file
    let tmp_path = "/tmp/keystone_deploy.tmp";
    if let Err(e) = fs::write(tmp_path, &payload.content).await {
        eprintln!("[API-App] Error in file creation: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DataJsonResponse {
                error: Some("Error in file creation".to_string()),
                data: None,
            }),
        );
    }

    // Find or create app in database
    let app = match find_or_create_app(&state.db_pool, &payload).await {
        Ok(app) => app,
        Err(e) => {
            eprintln!("[API-App] {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DataJsonResponse {
                    error: Some(format!("{}", e)),
                    data: None,
                }),
            );
        }
    };

    // Deploy app to IPFS
    let cid = match add_to_ipfs(&state.server_settings.server.ipfs_host, tmp_path).await {
        Ok(cid) => {
            println!("[API-App] File added to IPFS. CID: {}", cid);
            cid
        }
        Err(e) => {
            eprintln!("[API-App] Add to IPFS failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DataJsonResponse {
                    error: Some(e.to_string()),
                    data: None,
                }),
            );
        }
    };
    let deployment = match Deployment::create(
        &state.db_pool,
        &CreateDeploymentPayload {
            app_id: app.id.to_string(),
            cid: cid.clone(),
        },
    )
    .await
    {
        Ok(deployment) => deployment,
        Err(e) => {
            eprintln!("[API-App] Error in database deployment creation: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DataJsonResponse {
                    error: Some("Error in deployment creation".to_string()),
                    data: None,
                }),
            );
        }
    };

    // Find or create IPNS key
    let key_info =
        match find_or_create_ipns_key(&state.server_settings.server.ipfs_host, &app.name).await {
            Ok(info) => {
                println!("[API-App] IPNS key: {}", info.id);
                info
            }
            Err(e) => {
                eprintln!("[API-App] IPNS management failed: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DataJsonResponse {
                        error: Some(e.to_string()),
                        data: None,
                    }),
                );
            }
        };
    let app = match app
        .update(
            &state.db_pool,
            &UpdateAppPayload {
                team_id: None,
                name: None,
                key_name: Some(key_info.name.clone()),
                ipns_name: None,
            },
        )
        .await
    {
        Ok(app) => app,
        Err(e) => {
            eprintln!("[API-App] Error in database app update: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DataJsonResponse {
                    error: Some(format!("Error in app update: {}", e)),
                    data: None,
                }),
            );
        }
    };

    // Deploy to IPNS in background task
    let deployment = match deployment
        .update(
            &state.db_pool,
            &UpdateDeploymentPayload {
                app_id: None,
                cid: None,
                status: Some(DeploymentStatus::PUBLISHING),
            },
        )
        .await
    {
        Ok(deployment) => deployment,
        Err(e) => {
            eprintln!("[API-App] Error in database deployment update: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DataJsonResponse {
                    error: Some(format!("Error in deployment update: {}", e)),
                    data: None,
                }),
            );
        }
    };
    let db_pool_clone = state.db_pool.clone();
    let app_clone = app.clone();
    let ipfs_host_clone = state.server_settings.server.ipfs_host.clone();
    let key_name_clone = key_info.name.clone();
    let cid_clone = cid.clone();
    let deployment_clone = deployment.clone();
    tokio::spawn(async move {
        match publish_to_ipns(&ipfs_host_clone, &key_name_clone, &cid_clone).await {
            Ok(ipns_result) => {
                println!(
                    "[API-App] App \"{}\" published on IPNS ({} -> {})",
                    app_clone.name, ipns_result.name, ipns_result.value
                );
                let _ = app_clone
                    .update(
                        &db_pool_clone,
                        &UpdateAppPayload {
                            team_id: None,
                            name: None,
                            key_name: None,
                            ipns_name: Some(ipns_result.name.clone()),
                        },
                    )
                    .await;
                let _ = deployment_clone
                    .update(
                        &db_pool_clone,
                        &UpdateDeploymentPayload {
                            app_id: None,
                            cid: None,
                            status: Some(DeploymentStatus::DEPLOYED),
                        },
                    )
                    .await;
            }
            Err(e) => {
                eprintln!("[API-App] IPNS publication failed: {}", e);
                let _ = deployment_clone
                    .update(
                        &db_pool_clone,
                        &UpdateDeploymentPayload {
                            app_id: None,
                            cid: None,
                            status: Some(DeploymentStatus::FAILED),
                        },
                    )
                    .await;
            }
        };
    });

    let client = Client::new();
    let nodes_to_deploy = match select_nodes_deployable(&state).await {
        Ok(nodes_map) => {
            if nodes_map.is_empty() {
                println!("[API-App] App deployed, but no active node found to pin it.");
            }
            nodes_map
        }
        Err(e) => {
            eprintln!("[API-App] Error in retrieving nodes: {}.", e);
            HashMap::new()
        }
    };

    for (id, node) in nodes_to_deploy {
        let client_clone = client.clone();
        let deploy_url = format!("http://{}:{}/api/deploy", node.ip, node.port);
        let node_payload = NodeDeployPayload {
            name: app.name.clone(),
            cid: cid.clone(),
        };
        let state_clone = state.clone();
        let deployment_clone = deployment.clone();

        tokio::spawn(async move {
            let deployment_node = match DeploymentNode::create(
                &state_clone.db_pool,
                &CreateDeploymentNodePayload {
                    deployment_id: deployment_clone.id.to_string(),
                    node_id: id.clone(),
                },
            )
            .await
            {
                Ok(deployment_node) => deployment_node,
                Err(e) => {
                    eprintln!("[API-App] Error creating deployment_node record: {}", e);
                    return;
                }
            };

            match client_clone
                .post(&deploy_url)
                .json(&node_payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("[API-App] Send app deployment to node: id={}", id);
                        let _ = deployment_node
                            .update(
                                &state_clone.db_pool,
                                &UpdateDeploymentNodePayload {
                                    deployment_id: None,
                                    node_id: None,
                                    status: Some(PinStatus::PINNED),
                                },
                            )
                            .await;
                    } else {
                        println!(
                            "[API-App] Failed to app deployment to node: id={}, status={}",
                            id,
                            response.status()
                        );
                        let _ = deployment_node
                            .update(
                                &state_clone.db_pool,
                                &UpdateDeploymentNodePayload {
                                    deployment_id: None,
                                    node_id: None,
                                    status: Some(PinStatus::FAILED),
                                },
                            )
                            .await;
                    }
                }
                Err(e) => {
                    println!(
                        "[API-App] Error sending app deployment to node: id={}, error={}",
                        id, e
                    );
                    let _ = deployment_node
                        .update(
                            &state_clone.db_pool,
                            &UpdateDeploymentNodePayload {
                                deployment_id: None,
                                node_id: None,
                                status: Some(PinStatus::FAILED),
                            },
                        )
                        .await;
                }
            }
        });
    }

    (
        StatusCode::OK,
        Json(DataJsonResponse {
            data: Some(app),
            error: None,
        }),
    )
}

async fn find_or_create_app(db_pool: &DbPool, payload: &AppDeployPayload) -> Result<App, String> {
    if let Some(id) = &payload.id {
        match App::find_by_id(db_pool, id).await {
            Ok(app) => Ok(app),
            Err(e) => Err(format!("Error finding app by id in database: {}", e)),
        }
    } else {
        let team_id = match &payload.team_id {
            Some(tid) => tid.clone(),
            None => {
                return Err("team_id is required when creating a new app".to_string());
            }
        };
        let name = match &payload.name {
            Some(n) => n.clone(),
            None => {
                return Err("name is required when creating a new app".to_string());
            }
        };

        match App::create(
            db_pool,
            &CreateAppPayload {
                team_id: team_id,
                name: name,
            },
        )
        .await
        {
            Ok(app) => Ok(app),
            Err(e) => Err(format!("Error creating app in database: {}", e)),
        }
    }
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

async fn select_nodes_deployable(state: &ServerState) -> Result<HashMap<String, Node>, String> {
    match sqlx::query_as::<_, Node>("SELECT * FROM nodes WHERE reputation_score > 0.8 LIMIT 3")
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(nodes) => {
            let mut nodes_map = HashMap::new();
            for node in nodes {
                nodes_map.insert(node.id.to_string(), node);
            }
            Ok(nodes_map)
        }
        Err(e) => Err(format!("Error selecting nodes from database: {}", e)),
    }
}
