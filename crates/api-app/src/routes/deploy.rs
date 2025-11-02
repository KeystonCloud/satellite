use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use gw_core::{json::SimpleJsonResponse, node::NodeRegistry};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppDeployPayload {
    name: String,
}

pub async fn post(
    State(registry): State<NodeRegistry>,
    Json(payload): Json<AppDeployPayload>,
) -> impl IntoResponse {
    let client = Client::new();
    let nodes_to_deploy = registry.lock().unwrap().clone(); // TODO: filter nodes based on criteria (geo, capacity, reputation...)

    // EXAMPLE : send deployment on all nodes
    for (id, node) in nodes_to_deploy {
        let client_clone = client.clone();
        let deploy_url = format!("http://{}:{}/api/deploy", node.ip, node.port);
        let payload_clone = payload.clone();

        tokio::spawn(async move {
            let res = client_clone
                .post(&deploy_url)
                .json(&payload_clone)
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
            message: format!("Deploy request for app {} received", payload.name),
        }),
    )
}
