use crate::{error::Error, middleware::Server};

// TODO: Implement resource-based load balancing algorithm
pub async fn _resource_based(available_servers: &[Server]) -> Result<Server, Error> {
    Ok(available_servers[0].clone())
}
