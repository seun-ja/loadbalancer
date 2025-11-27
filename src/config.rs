use serde::Deserialize;

use crate::{
    algorithms::Algorithm,
    middleware::{Server, ServerClients},
};

#[derive(Deserialize, Debug)]
pub struct SystemConfig {
    pub available_servers: String, // TODO: This should be hosted in redis
    pub port: u16,
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
    pub available_servers: ServerClients,
    pub algorithm: Algorithm,
}

impl State {
    pub fn new(config: &SystemConfig) -> anyhow::Result<Self> {
        let servers = config.available_servers.split(',').collect::<Vec<&str>>();

        let available_servers: Vec<Server> = servers
            .clone()
            .into_iter()
            .map(Server::new)
            .collect::<anyhow::Result<Vec<Server>>>()?;

        Ok(State {
            available_servers: ServerClients::new(available_servers),
            algorithm: config.algorithm.clone().into(),
        })
    }
}
