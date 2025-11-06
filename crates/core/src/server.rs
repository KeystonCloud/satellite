use serde::Deserialize;

use crate::{
    app::AppRegistry,
    database::{DatabaseConfig, DbPool},
    node::NodeHealthConfig,
    redis::{RedisClient, RedisSettings},
};

#[derive(Clone)]
pub struct ServerState {
    pub server_settings: ServerSettings,
    pub app_registry: AppRegistry,
    pub db_pool: DbPool,
    pub redis_client: RedisClient,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub peer_id: String,
    pub ipfs_host: String,
    pub jwt_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub server: ServerConfig,
    pub node_health: NodeHealthConfig,
    pub database: DatabaseConfig,
    pub redis: RedisSettings,
}

impl ServerSettings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let config_builder = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::Environment::with_prefix("KC").separator("__"))
            .build()?;

        config_builder.try_deserialize()
    }
}
