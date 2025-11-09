use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use sqlx::{
    QueryBuilder,
    prelude::{FromRow, Type},
    types::Uuid,
};
use struct_iterable::Iterable;

use crate::payloads::deployment::{CreateDeploymentPayload, UpdateDeploymentPayload};
use kc_core::database::DbPool;

#[derive(Debug, Type, Serialize, Deserialize, Clone, Copy)]
#[sqlx(type_name = "deployment_status")]
pub enum DeploymentStatus {
    #[sqlx(rename = "PENDING")]
    PENDING,
    #[sqlx(rename = "PUBLISHING")]
    PUBLISHING,
    #[sqlx(rename = "DEPLOYED")]
    DEPLOYED,
    #[sqlx(rename = "FAILED")]
    FAILED,
}

#[derive(FromRow, Debug, Clone)]
pub struct Deployment {
    pub id: Uuid,
    pub app_id: Uuid,
    pub cid: String,
    pub status: DeploymentStatus,
    pub created_at: DateTime<Utc>,
}

impl Serialize for Deployment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Deployment", 5)?;
        state.serialize_field("id", &self.id.to_string())?;
        state.serialize_field("app_id", &self.app_id.to_string())?;
        state.serialize_field("cid", &self.cid)?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field("created_at", &self.created_at.to_string())?;
        state.end()
    }
}

impl Deployment {
    pub async fn create(
        db_pool: &DbPool,
        payload: &CreateDeploymentPayload,
    ) -> Result<Deployment, String> {
        match Uuid::parse_str(&payload.app_id) {
            Ok(app_id) => {
                match sqlx::query_as::<_, Deployment>(
                    "INSERT INTO deployments (app_id, cid) VALUES ($1, $2) RETURNING *",
                )
                .bind(app_id)
                .bind(payload.cid.clone())
                .fetch_one(db_pool)
                .await
                {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(format!("Invalid UUID format for app_id: {}", e)),
        }
    }

    pub async fn find_by_id(db_pool: &DbPool, id: &String) -> Result<Deployment, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, Deployment>("SELECT * FROM deployments WHERE id = $1")
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
        payload: &UpdateDeploymentPayload,
    ) -> Result<Deployment, String> {
        deployment_update_by_id(db_pool, id, payload).await
    }

    pub async fn update(
        &self,
        db_pool: &DbPool,
        payload: &UpdateDeploymentPayload,
    ) -> Result<Deployment, String> {
        deployment_update_by_id(db_pool, &self.id.to_string(), payload).await
    }

    pub async fn delete_by_id(db_pool: &DbPool, id: &String) -> Result<Deployment, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, Deployment>(
                    "DELETE FROM deployments WHERE id = $1 RETURNING *",
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

async fn deployment_update_by_id(
    db_pool: &DbPool,
    id: &String,
    payload: &UpdateDeploymentPayload,
) -> Result<Deployment, String> {
    match Uuid::parse_str(id) {
        Ok(uuid) => {
            let mut query_builder = QueryBuilder::new("UPDATE deployments");

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

                if let Some(value) = field_value.downcast_ref::<Option<DeploymentStatus>>() {
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

            let query = query_builder.build_query_as::<Deployment>();

            match query.fetch_one(db_pool).await {
                Ok(result) => Ok(result),
                Err(e) => Err(e.to_string()),
            }
        }
        Err(e) => Err(format!("Invalid UUID format: {}", e)),
    }
}
