use crate::{error::Error, middleware::Server};

pub async fn weighted_least_connection(available_servers: &[Server]) -> Result<Server, Error> {
    Ok(available_servers[0].clone())
}
