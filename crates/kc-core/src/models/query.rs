use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};
use sqlx::types::Uuid;

use crate::{authentication::Claims, models::user::User, server::ServerState};

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;
pub struct Query;

#[Object]
impl Query {
    async fn hello(&self) -> &'static str {
        "world"
    }

    async fn me(&self, ctx: &Context<'_>) -> Result<User, String> {
        let state = match ctx.data::<ServerState>() {
            Ok(state) => state,
            Err(_) => {
                return Err("Failed to get server state".to_string());
            }
        };

        let claims = match ctx.data::<Claims>() {
            Ok(claims) => claims,
            Err(e) => {
                println!("{:?}", e);
                return Err("User not connected".to_string());
            }
        };

        println!("Claims: {:?}", claims);

        match Uuid::parse_str(&claims.user_id) {
            Ok(uuid) => {
                match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                    .bind(uuid)
                    .fetch_one(&state.db_pool)
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

pub fn build_schema() -> AppSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}
