use crate::{error::Error, middleware::Server};

pub async fn resource_based(_available_servers: &[Server]) -> Result<Server, Error> {
    todo!()
}
