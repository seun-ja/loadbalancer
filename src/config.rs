use serde::Deserialize;

use crate::{
    db::{self, RedisClient},
    middleware::{Server, ServerClients},
};

#[derive(Deserialize, Debug)]
pub struct SystemConfig {
    pub available_servers: String, // TODO: This should be hosted in redis
    pub port: u16,
    pub redis_url: String,
}

impl SystemConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv_override().ok();

        envy::from_env::<Self>()
            .map_err(|e| anyhow::anyhow!("Failed to load environment variables: {}", e))
    }
}

#[derive(Clone)]
pub struct State {
    pub available_servers: ServerClients,
    pub redis_conn: RedisClient,
}

impl State {
    pub async fn new(config: &SystemConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let servers = config.available_servers.split(',').collect::<Vec<&str>>();

        let available_servers: Vec<Server> = servers
            .clone()
            .into_iter()
            .map(Server::new)
            .collect::<Result<Vec<Server>, _>>()?;

        let redis_conn =
            db::RedisClient::init_redis(&config.redis_url, available_servers.clone()).await?;

        Ok(State {
            available_servers: ServerClients::new(available_servers),
            redis_conn,
        })
    }
}
