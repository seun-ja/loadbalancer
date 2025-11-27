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
