use serde::Deserialize;
use struct_iterable::Iterable;

use crate::models::deployment_node::PinStatus;

#[derive(Deserialize, Debug)]
pub struct CreateDeploymentNodePayload {
    pub deployment_id: String,
    pub node_id: String,
    pub status: PinStatus,
}

#[derive(Deserialize, Debug, Iterable)]
pub struct UpdateDeploymentNodePayload {
    pub deployment_id: Option<String>,
    pub node_id: Option<String>,
    pub status: Option<PinStatus>,
}
