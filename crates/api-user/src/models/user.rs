use chrono::{DateTime, Utc};
use serde::ser::{Serialize, SerializeStruct};
use sqlx::{QueryBuilder, prelude::FromRow, types::Uuid};
use struct_iterable::Iterable;

use crate::{
    models::team::Team,
    payloads::user::{CreateUserPayload, LoginPayload, UpdateUserPayload},
    utils::auth::{hash_password, verify_password},
};
use kc_core::database::DbPool;

#[derive(FromRow, Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("User", 5)?;
        state.serialize_field("id", &self.id.to_string())?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("email", &self.email)?;
        state.serialize_field("role", &self.role)?;
        state.serialize_field("created_at", &self.created_at.to_string())?;
        state.serialize_field("updated_at", &self.updated_at.to_string())?;
        state.end()
    }
}

impl User {
    pub async fn create(db_pool: &DbPool, payload: &CreateUserPayload) -> Result<User, String> {
        let password_hash = match hash_password(payload.password.clone()).await {
            Ok(hash) => hash,
            Err(e) => {
                eprintln!("Error in hashing password: {}", e);
                return Err("Error in hashing password".to_string());
            }
        };

        match sqlx::query_as::<_, User>(
            "INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(payload.name.clone())
        .bind(payload.email.clone())
        .bind(password_hash)
        .fetch_one(db_pool)
        .await
        {
            Ok(result) => Ok(result),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn find_by_id(db_pool: &DbPool, id: &String) -> Result<User, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
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
        payload: &UpdateUserPayload,
    ) -> Result<User, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                let mut query_builder = QueryBuilder::new("UPDATE users");

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

                let query = query_builder.build_query_as::<User>();

                match query.fetch_one(db_pool).await {
                    Ok(result) => Ok(result),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(format!("Invalid UUID format: {}", e)),
        }
    }

    pub async fn delete_by_id(db_pool: &DbPool, id: &String) -> Result<User, String> {
        match Uuid::parse_str(id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, User>("DELETE FROM users WHERE id = $1 RETURNING *")
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

    pub async fn login(db_pool: &DbPool, payload: &LoginPayload) -> Result<User, String> {
        match sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(payload.email.clone())
            .fetch_one(db_pool)
            .await
        {
            Ok(result) => {
                match verify_password(payload.password.clone(), result.password.clone()).await {
                    Ok(is_valid) => {
                        if is_valid {
                            return Ok(result);
                        }

                        Err("Invalid credentials".to_string())
                    }
                    Err(e) => {
                        eprintln!("Error in password verification: {}", e);
                        return Err("Error in password verification".to_string());
                    }
                }
            }
            Err(e) => Err(format!("User not found: {}", e)),
        }
    }

    pub async fn associate_team(&self, db_pool: &DbPool, team: &Team) -> Result<(), String> {
        match sqlx::query("INSERT INTO team_users (team_id, user_id) VALUES ($1, $2)")
            .bind(team.id)
            .bind(self.id)
            .execute(db_pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}
