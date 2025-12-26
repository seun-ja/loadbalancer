use crate::{error::Error, middleware::StaticServerData};

// TODO: Implement resource-based load balancing algorithm
pub async fn _resource_based(
    available_servers: &[StaticServerData],
) -> Result<StaticServerData, Error> {
    Ok(available_servers[0].clone())
}
