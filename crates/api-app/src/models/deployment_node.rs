use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use sqlx::{
    QueryBuilder,
    prelude::{FromRow, Type},
    types::Uuid,
};
use struct_iterable::Iterable;

use crate::payloads::deployment_node::{CreateDeploymentNodePayload, UpdateDeploymentNodePayload};
use core::database::DbPool;

#[derive(Debug, Type, Serialize, Deserialize, Clone, Copy)]
#[sqlx(type_name = "deployment_status")]
pub enum PinStatus {
    #[sqlx(rename = "PINNING")]
    PINNING,
    #[sqlx(rename = "PINNED")]
    PINNED,
    #[sqlx(rename = "FAILED")]
    FAILED,
}

#[derive(FromRow, Debug)]
pub struct DeploymentNode {
    pub id: Uuid,
    pub deployment_id: Uuid,
    pub node_id: Uuid,
    pub status: PinStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Serialize for DeploymentNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("DeploymentNode", 5)?;
        state.serialize_field("id", &self.id.to_string())?;
        state.serialize_field("deployment_id", &self.deployment_id.to_string())?;
        state.serialize_field("node_id", &self.node_id.to_string())?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field("created_at", &self.created_at.to_string())?;
        state.serialize_field("updated_at", &self.updated_at.to_string())?;
        state.end()
    }
}

impl DeploymentNode {
    pub async fn create(
        db_pool: &DbPool,
        payload: &CreateDeploymentNodePayload,
    ) -> Result<DeploymentNode, String> {
        match sqlx::query_as::<_, DeploymentNode>(
            "INSERT INTO deployments_nodes (deployment_id, node_id, status) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(payload.deployment_id.clone())
        .bind(payload.node_id.clone())
        .bind(payload.status)
        .fetch_one(db_pool)
        .await
        {
            Ok(result) => Ok(result),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn find_by_id(db_pool: &DbPool, id: &String) -> Result<DeploymentNode, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, DeploymentNode>(
                    "SELECT * FROM deployments_nodes WHERE id = $1",
                )
                .bind(uuid)
                .fetch_one(db_pool)
                .await
                {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(format!("Invalid UUID format: {}", e)),
        }
    }

    pub async fn update_by_id(
        db_pool: &DbPool,
        id: &String,
        payload: &UpdateDeploymentNodePayload,
    ) -> Result<DeploymentNode, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                let mut query_builder = QueryBuilder::new("UPDATE deployments_nodes");

                let mut i = 0;
                for (name, field_value) in payload.iter() {
                    if let Some(value) = field_value.downcast_ref::<Option<String>>() {
                        if let Some(v) = value {
                            if i == 0 {
                                query_builder.push(" SET ");
                            } else {
                                query_builder.push(", ");
                            }

                            query_builder.push(name).push(" = ").push_bind(v);
                            i += 1;
                        }
                    }
                }

                query_builder.push(" WHERE id = ").push_bind(uuid);
                query_builder.push(" RETURNING *");

                let query = query_builder.build_query_as::<DeploymentNode>();

                match query.fetch_one(db_pool).await {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(format!("Invalid UUID format: {}", e)),
        }
    }

    pub async fn delete_by_id(db_pool: &DbPool, id: &String) -> Result<DeploymentNode, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, DeploymentNode>(
                    "DELETE FROM deployments_nodes WHERE id = $1 RETURNING *",
                )
                .bind(uuid)
                .fetch_one(db_pool)
                .await
                {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(format!("Invalid UUID format: {}", e)),
        }
    }
}
