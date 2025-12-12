use crate::{error::Error, middleware::Server};

mod least_connection;
mod resource_based;
mod weighted_least_connection;
mod weighted_response_time;

#[derive(Clone, Default)]
pub enum Algorithm {
    #[default]
    LeastConnection,
    ResourceBased,
    WeightedLeastConnection,
    WeightedResponseTime,
}

impl From<String> for Algorithm {
    fn from(algorithm: String) -> Self {
        match algorithm.as_str() {
            "least_connection" => Algorithm::LeastConnection,
            "resource_based" => Algorithm::ResourceBased,
            "weighted_least_connection" => Algorithm::WeightedLeastConnection,
            "weighted_response_time" => Algorithm::WeightedResponseTime,
            _ => Algorithm::default(),
        }
    }
}

impl Algorithm {
    pub async fn select_server(&self, available_servers: &[Server]) -> Result<Server, Error> {
        match self {
            Algorithm::LeastConnection => {
                least_connection::least_connection(available_servers).await
            }
            Algorithm::ResourceBased => resource_based::resource_based(available_servers).await,
            Algorithm::WeightedLeastConnection => {
                weighted_least_connection::weighted_least_connection(available_servers).await
            }
            Algorithm::WeightedResponseTime => {
                weighted_response_time::weighted_response_time(available_servers).await
            }
        }
    }
}
