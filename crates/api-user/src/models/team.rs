use chrono::{DateTime, Utc};
use serde::ser::{Serialize, SerializeStruct};
use sqlx::{QueryBuilder, prelude::FromRow, types::Uuid};
use struct_iterable::Iterable;

use crate::{
    models::user::User,
    payloads::team::{CreateTeamPayload, UpdateTeamPayload},
};
use core::database::DbPool;

#[derive(FromRow, Debug)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Serialize for Team {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Team", 5)?;
        state.serialize_field("id", &self.id.to_string())?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("created_at", &self.created_at.to_string())?;
        state.serialize_field("updated_at", &self.updated_at.to_string())?;
        state.end()
    }
}

impl Team {
    pub async fn create(db_pool: &DbPool, payload: &CreateTeamPayload) -> Result<Team, String> {
        match sqlx::query_as::<_, Team>("INSERT INTO teams (name) VALUES ($1) RETURNING *")
            .bind(payload.name.clone())
            .fetch_one(db_pool)
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn find_by_id(db_pool: &DbPool, id: &String) -> Result<Team, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, Team>("SELECT * FROM teams WHERE id = $1")
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
        payload: &UpdateTeamPayload,
    ) -> Result<Team, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                let mut query_builder = QueryBuilder::new("UPDATE teams");

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

                let query = query_builder.build_query_as::<Team>();

                match query.fetch_one(db_pool).await {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(format!("Invalid UUID format: {}", e)),
        }
    }

    pub async fn delete_by_id(db_pool: &DbPool, id: &String) -> Result<Team, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, Team>("DELETE FROM teams WHERE id = $1 RETURNING *")
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

    pub async fn associate_user(&self, db_pool: &DbPool, user: &User) -> Result<(), String> {
        match sqlx::query("INSERT INTO team_users (team_id, user_id) VALUES ($1, $2)")
            .bind(self.id)
            .bind(user.id)
            .execute(db_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}
