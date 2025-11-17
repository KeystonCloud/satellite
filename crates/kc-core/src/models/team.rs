use async_graphql::{Context, Object};
use chrono::{DateTime, Utc};
use serde::ser::{Serialize, SerializeStruct};
use sqlx::{QueryBuilder, prelude::FromRow, types::Uuid};
use struct_iterable::Iterable;

use crate::{
    database::DbPool,
    models::{app::App, node::Node, user::User},
    payloads::team::{CreateTeamPayload, UpdateTeamPayload},
    server::ServerState,
};

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

    pub async fn find_by_user_id(db_pool: &DbPool, id: &String) -> Result<Vec<Team>, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, Team>("SELECT * FROM teams JOIN team_users ON team_users.team_id = teams.id WHERE team_users.user_id = $1")
                    .bind(uuid)
                    .fetch_all(db_pool)
                    .await
                {
                    Ok(results) => Ok(results),
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

    pub async fn associate_user_by_id(
        &self,
        db_pool: &DbPool,
        user_id: &String,
    ) -> Result<(), String> {
        match sqlx::query("INSERT INTO team_users (team_id, user_id) VALUES ($1, $2)")
            .bind(self.id)
            .bind(user_id)
            .execute(db_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[Object]
impl Team {
    async fn id(&self) -> Uuid {
        self.id
    }
    async fn name(&self) -> &str {
        &self.name
    }

    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>, String> {
        let state = match ctx.data::<ServerState>() {
            Ok(state) => state,
            Err(_) => {
                return Err("Failed to get server state".to_string());
            }
        };

        match sqlx::query_as::<_, User>(
            "SELECT u.* FROM users u JOIN team_users tu ON u.id = tu.user_id WHERE tu.team_id = $1",
        )
        .bind(self.id)
        .fetch_all(&state.db_pool)
        .await
        {
            Ok(results) => Ok(results),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn nodes(&self, ctx: &Context<'_>) -> Result<Vec<Node>, String> {
        let state = match ctx.data::<ServerState>() {
            Ok(state) => state,
            Err(_) => {
                return Err("Failed to get server state".to_string());
            }
        };

        match sqlx::query_as::<_, Node>("SELECT * FROM nodes WHERE owner_id = $1")
            .bind(self.id)
            .fetch_all(&state.db_pool)
            .await
        {
            Ok(results) => Ok(results),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn apps(&self, ctx: &Context<'_>) -> Result<Vec<App>, String> {
        let state = match ctx.data::<ServerState>() {
            Ok(state) => state,
            Err(_) => {
                return Err("Failed to get server state".to_string());
            }
        };

        match sqlx::query_as::<_, App>("SELECT * FROM apps WHERE team_id = $1")
            .bind(self.id)
            .fetch_all(&state.db_pool)
            .await
        {
            Ok(results) => Ok(results),
            Err(e) => Err(e.to_string()),
        }
    }
}
