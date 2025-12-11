use crate::{error::Error, middleware::Server};

pub async fn resource_based(available_servers: &[Server]) -> Result<Server, Error> {
    Ok(available_servers[0].clone())
}
