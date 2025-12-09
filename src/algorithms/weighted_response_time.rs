use crate::{error::Error, middleware::Server};

pub async fn weighted_response_time(_available_servers: &[Server]) -> Result<Server, Error> {
    todo!()
}
