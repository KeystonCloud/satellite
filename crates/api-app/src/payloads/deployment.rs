use serde::Deserialize;
use struct_iterable::Iterable;

use crate::models::deployment::DeploymentStatus;

#[derive(Deserialize, Debug)]
pub struct CreateDeploymentPayload {
    pub app_id: String,
    pub cid: String,
    pub status: DeploymentStatus,
}

#[derive(Deserialize, Debug, Iterable)]
pub struct UpdateDeploymentPayload {
    pub app_id: Option<String>,
    pub cid: Option<String>,
    pub status: Option<DeploymentStatus>,
}
