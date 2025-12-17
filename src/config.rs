use serde::Deserialize;

use crate::{
    algorithms::Algorithm,
    db::{self, RedisClient},
    middleware::Server,
};

#[derive(Deserialize)]
pub struct SystemConfig {
    pub available_servers: String, // TODO: This should be hosted in redis
    pub port: u16,
    pub redis_url: String,
    pub algorithm: String,
    pub trace_level: String,
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
    pub redis_conn: RedisClient,
    pub algorithm: Algorithm,
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
            redis_conn,
            algorithm: config.algorithm.clone().into(),
        })
    }
}
