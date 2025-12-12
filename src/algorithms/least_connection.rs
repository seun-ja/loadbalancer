use crate::{error::Error, middleware::Server};

pub async fn least_connection(available_servers: &[Server]) -> Result<Server, Error> {
    let server = available_servers
        .iter()
        .min_by_key(|server| server.load())
        .ok_or_else(|| Error::NoServerAvailable)?;

    Ok(server.clone())
}
