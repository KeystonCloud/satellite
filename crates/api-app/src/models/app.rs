use chrono::{DateTime, Utc};
use serde::ser::{Serialize, SerializeStruct};
use sqlx::{QueryBuilder, prelude::FromRow, types::Uuid};
use struct_iterable::Iterable;

use crate::payloads::app::{CreateAppPayload, UpdateAppPayload};
use core::database::DbPool;

#[derive(FromRow, Debug)]
pub struct App {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub key_name: String,
    pub ipns_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Serialize for App {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("App", 5)?;
        state.serialize_field("id", &self.id.to_string())?;
        state.serialize_field("team_id", &self.team_id.to_string())?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("key_name", &self.key_name)?;
        state.serialize_field("ipns_name", &self.ipns_name)?;
        state.serialize_field("created_at", &self.created_at.to_string())?;
        state.serialize_field("updated_at", &self.updated_at.to_string())?;
        state.end()
    }
}

impl App {
    pub async fn create(db_pool: &DbPool, payload: &CreateAppPayload) -> Result<App, String> {
        match sqlx::query_as::<_, App>(
            "INSERT INTO apps (team_id, name, key_name, ipns_name) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(payload.team_id.clone())
        .bind(payload.name.clone())
        .bind(payload.key_name.clone())
        .bind(payload.ipns_name.clone())
        .fetch_one(db_pool)
        .await
        {
            Ok(result) => Ok(result),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn find_by_id(db_pool: &DbPool, id: &String) -> Result<App, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, App>("SELECT * FROM apps WHERE id = $1")
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
        payload: &UpdateAppPayload,
    ) -> Result<App, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                let mut query_builder = QueryBuilder::new("UPDATE apps");

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

                let query = query_builder.build_query_as::<App>();

                match query.fetch_one(db_pool).await {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(format!("Invalid UUID format: {}", e)),
        }
    }

    pub async fn delete_by_id(db_pool: &DbPool, id: &String) -> Result<App, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, App>("DELETE FROM apps WHERE id = $1 RETURNING *")
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
