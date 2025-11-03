use serde::Deserialize;

use crate::{
    app::AppRegistry,
    node::{NodeHealthConfig, NodeRegistry},
};

#[derive(Clone)]
pub struct ServerState {
    pub server_settings: ServerSettings,
    pub node_registry: NodeRegistry,
    pub app_registry: AppRegistry,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub peer_id: String,
    pub ipfs_host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub server: ServerConfig,
    pub node_health: NodeHealthConfig,
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
