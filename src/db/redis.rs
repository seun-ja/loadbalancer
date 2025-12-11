use redis::{AsyncTypedCommands as _, cluster::ClusterClient, cluster_async::ClusterConnection};

use crate::{error::Error, middleware::Server};

#[derive(Clone)]
pub struct RedisClient(ClusterConnection);

impl RedisClient {
    pub async fn init_redis(
        redis_url: &str,
        available_servers: Vec<Server>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let nodes = vec![redis_url];
        let client = ClusterClient::new(nodes)?;
        let mut connection = client.get_async_connection().await?;

        // Preload server urls into Redis
        for (server_index, server) in available_servers.clone().into_iter().enumerate() {
            connection
                .set(format!("server_{}", server_index), server.url.as_str())
                .await?;
        }

        Ok(Self(connection))
    }

    pub async fn set(&mut self, key: &str, value: &str) -> Result<(), Error> {
        Ok(self.0.set(key, value).await?)
    }

    pub async fn get(&mut self, key: &str) -> Result<Option<String>, Error> {
        Ok(self.0.get(key).await?)
    }

    pub async fn delete(&mut self, key: &str) -> Result<usize, Error> {
        Ok(self.0.del(key).await?)
    }
}
