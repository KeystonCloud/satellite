use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use sqlx::{QueryBuilder, prelude::FromRow, types::Uuid};
use struct_iterable::Iterable;

use crate::payloads::node::{CreateNodePayload, UpdateNodePayload};
use kc_core::database::DbPool;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeInfo {
    pub last_seen: Option<i64>,
}

#[derive(FromRow, Debug)]
pub struct Node {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub reputation_score: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct NodeData {
    pub node: Node,
    pub info: Option<NodeInfo>,
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Node", 5)?;
        state.serialize_field("id", &self.id.to_string())?;
        state.serialize_field("owner_id", &self.owner_id.to_string())?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("ip", &self.ip)?;
        state.serialize_field("port", &self.port)?;
        state.serialize_field("reputation_score", &self.reputation_score)?;
        state.serialize_field("created_at", &self.created_at.to_string())?;
        state.serialize_field("updated_at", &self.updated_at.to_string())?;
        state.end()
    }
}

impl Node {
    pub async fn create(db_pool: &DbPool, payload: &CreateNodePayload) -> Result<Node, String> {
        match Uuid::parse_str(payload.owner_id.as_str()) {
            Ok(owner_id) => {
                match sqlx::query_as::<_, Node>(
                    "INSERT INTO nodes (owner_id, name, ip, port) VALUES ($1, $2, $3, $4) RETURNING *",
                )
                .bind(owner_id)
                .bind(payload.name.clone())
                .bind(payload.ip.clone())
                .bind(payload.port)
                .fetch_one(db_pool)
                .await
                {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(format!("Invalid owner uuid format: {}", e)),
        }
    }

    pub async fn find_by_id(db_pool: &DbPool, id: &String) -> Result<Node, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, Node>("SELECT * FROM nodes WHERE id = $1")
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
        payload: &UpdateNodePayload,
    ) -> Result<Node, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                let mut query_builder = QueryBuilder::new("UPDATE nodes");

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

                let query = query_builder.build_query_as::<Node>();

                match query.fetch_one(db_pool).await {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(format!("Invalid UUID format: {}", e)),
        }
    }

    pub async fn delete_by_id(db_pool: &DbPool, id: &String) -> Result<Node, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, Node>("DELETE FROM nodes WHERE id = $1 RETURNING *")
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
