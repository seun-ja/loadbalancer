use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU32, AtomicU64},
    },
};

use redis::{AsyncTypedCommands as _, cluster::ClusterClient, cluster_async::ClusterConnection};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    middleware::{Server, ServerClient},
};

#[derive(Clone)]
pub struct RedisClient(ClusterConnection);

impl RedisClient {
    pub async fn init_redis(
        redis_url: &str,
        available_servers: Vec<Server>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let nodes = vec![redis_url];
        let client = ClusterClient::new(nodes)?;
        let connection = client.get_async_connection().await?;

        let mut client = Self(connection);

        // Preload server data into respective Redis keys
        for server in available_servers.clone().into_iter() {
            client
                .update_server_data(server.url.as_str(), &server.clone().static_data()?)
                .await?;

            client
                .update_server_load(server.url.as_str(), server.clone().load())
                .await?;

            client
                .update_server_mean_latency_record(
                    &format!("{}_latency", server.url.as_str()),
                    server.mean_latency(),
                )
                .await?;
        }

        Ok(client)
    }

    // Basic Commands

    /// Set a key-value pair in Redis.
    pub async fn set(&mut self, key: &str, value: &str) -> Result<(), Error> {
        Ok(self.0.set(key, value).await?)
    }

    /// Get the value associated with a key from Redis.
    pub async fn get(&mut self, key: &str) -> Result<Option<String>, Error> {
        Ok(self.0.get(key).await?)
    }

    /// Delete a key from Redis.
    pub async fn delete(&mut self, key: &str) -> Result<usize, Error> {
        Ok(self.0.del(key).await?)
    }

    // Server Data Commands

    /// Update the data of a server in Redis.
    pub async fn update_server_data(&mut self, key: &str, value: &str) -> Result<(), Error> {
        Ok(self.0.hset("server", key, value).await.map(|_| ())?)
    }

    /// Get the data of a server from Redis.
    pub async fn get_server_data(&mut self, key: &str) -> Result<Option<ServerClient>, Error> {
        self.0
            .hget("server", key)
            .await
            .map(|d| d.map(StaticServerData::from_json))?
            .transpose()
            .map(|t| t.map(|s| s.into()))
    }

    /// Get all server data from Redis.
    pub async fn get_all_server_data(&mut self) -> Result<HashMap<String, ServerClient>, Error> {
        self.0
            .hgetall("server")
            .await?
            .into_iter()
            .map(|(k, v)| Ok((k, StaticServerData::from_json(v)?.into())))
            .collect::<Result<HashMap<_, _>, _>>()
    }

    // Server Load Commands

    /// Update the load of a server in Redis.
    pub async fn update_server_load(&mut self, key: &str, value: u32) -> Result<(), Error> {
        Ok(self.0.hset("server_load", key, value).await.map(|_| ())?)
    }

    /// Get the load of a server from Redis.
    pub async fn get_server_load(&mut self, key: &str) -> Result<Option<u32>, Error> {
        self.0
            .hget("server_load", key)
            .await
            .map_err(Error::RedisError)?
            .map(|d| d.parse::<u32>().map_err(Error::ParseIntError))
            .transpose()
    }

    /// Get all server load data from Redis.
    pub async fn get_all_server_load(&mut self) -> Result<HashMap<String, u32>, Error> {
        self.0
            .hgetall("server_load")
            .await?
            .into_iter()
            .map(|(k, v)| Ok((k, v.parse::<u32>().map_err(Error::ParseIntError)?)))
            .collect::<Result<HashMap<_, _>, _>>()
    }

    // Latency Commands

    /// Update the mean latency record of a server in Redis.
    pub async fn update_server_mean_latency_record(
        &mut self,
        key: &str,
        value: u128,
    ) -> Result<(), Error> {
        Ok(self.0.rpush(key, value).await.map(|_| ())?)
    }

    /// Get the mean latency record of all servers in Redis.
    pub async fn get_servers_mean_latency_record(
        &mut self,
    ) -> Result<HashMap<String, Vec<u128>>, Error> {
        todo!()
    }

    /// Update the mean latency of a server in Redis.
    pub async fn update_server_mean_latency(
        &mut self,
        key: &str,
        value: u128,
    ) -> Result<(), Error> {
        Ok(self
            .0
            .hset("server_latency", key, value)
            .await
            .map(|_| ())?)
    }

    /// Get the mean latency of all servers in Redis.
    pub async fn get_all_server_mean_latency(&mut self) -> Result<HashMap<String, u32>, Error> {
        self.0
            .hgetall("server_latency")
            .await?
            .into_iter()
            .map(|(k, v)| Ok((k, v.parse::<u32>().map_err(Error::ParseIntError)?)))
            .collect::<Result<HashMap<_, _>, _>>()
    }

    // Weights Commands

    /// Update the weight of a server in Redis.
    pub async fn update_server_weight(&mut self, key: &str, value: u32) -> Result<(), Error> {
        Ok(self
            .0
            .hset("server_weights", key, value)
            .await
            .map(|_| ())?)
    }

    /// Get the weights of all servers in Redis.
    pub async fn get_all_server_weights(&mut self) -> Result<HashMap<String, u32>, Error> {
        self.0
            .hgetall("server_weights")
            .await?
            .into_iter()
            .map(|(k, v)| Ok((k, v.parse::<u32>().map_err(Error::ParseIntError)?)))
            .collect::<Result<HashMap<_, _>, _>>()
    }
}

#[derive(Serialize, Deserialize)]
pub struct StaticServerData {
    pub url: Url,
    pub weight: u32,
}

impl StaticServerData {
    pub fn from_json(data: String) -> Result<Self, Error> {
        serde_json::from_str(&data).map_err(Error::SerializationError)
    }
}

pub struct ServerInfo {
    pub key: String,
    pub url: Url,
    pub load: Arc<AtomicU32>,
    pub weight: u32,
    pub mean_latency: Arc<AtomicU64>,
}

impl ServerInfo {
    pub fn key(&self) -> &str {
        &self.key
    }
}
