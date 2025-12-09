use crate::{error::Error, middleware::Server};

pub async fn weighted_least_connection(_available_servers: &[Server]) -> Result<Server, Error> {
    todo!()
}
