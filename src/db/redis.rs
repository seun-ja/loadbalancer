use std::collections::HashMap;

use redis::{AsyncTypedCommands as _, cluster::ClusterClient, cluster_async::ClusterConnection};

use crate::{
    error::Error,
    middleware::{ServerClient, StaticServerData},
};

#[derive(Clone)]
pub struct RedisClient(ClusterConnection);

impl RedisClient {
    pub async fn init_redis(
        redis_url: &str,
        available_servers: Vec<StaticServerData>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let nodes = vec![redis_url];
        let client = ClusterClient::new(nodes)?;
        let connection = client.get_async_connection().await?;

        let mut client = Self(connection);

        // Preload server data into respective Redis keys
        for server in available_servers.clone().into_iter() {
            client.update_server_url(server.url.as_str()).await?;

            client
                .update_server_weight(server.url.as_str(), server.weight)
                .await?
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
    pub async fn update_server_url(&mut self, value: &str) -> Result<(), Error> {
        Ok(self.0.rpush("server_url", value).await.map(|_| ())?)
    }

    /// Get all server data from Redis.
    pub async fn get_all_server_url(&mut self) -> Result<Vec<ServerClient>, Error> {
        self.0
            .lrange("server_url", 0, -1)
            .await?
            .into_iter()
            .map(|v| Ok(StaticServerData::from_json(v)?.into()))
            .collect::<Result<Vec<_>, _>>()
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
    pub async fn update_server_latency_record(
        &mut self,
        key: &str,
        value: u128,
    ) -> Result<(), Error> {
        Ok(self.0.rpush(key, value).await.map(|_| ())?)
    }

    pub async fn get_server_latency_record(&mut self, key: &str) -> Result<Vec<u128>, Error> {
        self.0
            .lrange(key, 0, -1)
            .await?
            .into_iter()
            .map(|v| Ok(v.parse()?))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Get the latency record of all servers in Redis.
    pub async fn get_servers_latency_record(
        &mut self,
    ) -> Result<HashMap<String, Vec<u128>>, Error> {
        let server_client = self.get_all_server_url().await?;

        let mut res: HashMap<String, Vec<u128>> = HashMap::new();

        for server in server_client {
            let latencies = self.get_server_latency_record(server.url.as_str()).await?;
            res.insert(server.url.to_string(), latencies);
        }
        Ok(res)
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
