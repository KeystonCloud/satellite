use redis::{Client, RedisError};
use serde::Deserialize;

pub type RedisClient = Client;

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

impl RedisSettings {
    pub fn url(&self) -> String {
        format!(
            "redis://{}:{}@{}:{}",
            self.user, self.password, self.host, self.port
        )
    }

    pub fn create_client(&self) -> Result<RedisClient, RedisError> {
        let url = self.url();
        match RedisClient::open(url) {
            Ok(client) => Ok(client),
            Err(e) => Err(e),
        }
    }
}
