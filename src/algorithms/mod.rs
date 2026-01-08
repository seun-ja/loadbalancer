use reqwest::Url;

use crate::{db::RedisClient, error::Error, middleware::ServerClient};

mod least_connection;
mod location_based;
mod resource_based;
mod weighted_least_connection;
mod weighted_response_time;

#[derive(Clone, Default)]
pub enum Algorithm {
    #[default]
    LeastConnection,
    LocationBased,
    ResourceBased,
    WeightedLeastConnection,
    WeightedResponseTime,
}

impl From<String> for Algorithm {
    fn from(algorithm: String) -> Self {
        match algorithm.as_str() {
            "least_connection" => Algorithm::LeastConnection,
            "location_based" | "location" => Algorithm::LocationBased,
            "resource_based" => Algorithm::ResourceBased,
            "weighted_least_connection" => Algorithm::WeightedLeastConnection,
            "weighted_response_time" => Algorithm::WeightedResponseTime,
            _ => Algorithm::default(),
        }
    }
}

impl Algorithm {
    pub async fn select_server(
        &self,
        mut redis_client: RedisClient,
        location: &str,
    ) -> Result<ServerClient, Error> {
        let server_loads = redis_client.get_all_server_load().await?;
        let weights = redis_client.get_all_server_weights().await?;

        let url = match self {
            Algorithm::LeastConnection => least_connection::least_connection(server_loads).await,
            Algorithm::LocationBased => location_based::location_based(location).await,
            Algorithm::ResourceBased => unimplemented!(),
            Algorithm::WeightedLeastConnection => {
                weighted_least_connection::weighted_least_connection(server_loads, weights).await
            }
            Algorithm::WeightedResponseTime => {
                weighted_response_time::weighted_response_time(
                    redis_client.get_all_server_mean_latency().await?,
                    weights,
                )
                .await
            }
        }?;

        redis_client.update_server_load(&url, 1).await?;

        let url = url.parse::<Url>().map_err(|e| Error::Other(e.into()))?;

        Ok(ServerClient {
            url,
            client: reqwest::Client::new(),
        })
    }
}
