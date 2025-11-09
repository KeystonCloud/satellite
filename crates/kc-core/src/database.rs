use serde::Deserialize;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub connection: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub pool_size: u32,
}

#[derive(Debug)]
pub struct DbError {
    pub message: String,
}

pub type DbPool = PgPool;

pub async fn create_db_pool(config: &DatabaseConfig) -> Result<DbPool, DbError> {
    if config.connection != "postgres" {
        return Err(DbError {
            message: format!(
                "Unsupported database connection type: {}",
                config.connection
            ),
        });
    }

    match PgPoolOptions::new()
        .max_connections(5)
        .connect(
            format!(
                "{}://{}:{}@{}:{}/{}",
                config.connection,
                config.username,
                config.password,
                config.host,
                config.port,
                config.database_name
            )
            .as_str(),
        )
        .await
    {
        Ok(pool) => Ok(pool),
        Err(e) => Err(DbError {
            message: format!("Failed to create database pool: {}", e),
        }),
    }
}
