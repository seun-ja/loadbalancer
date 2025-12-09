// Server reports its current load
// Redis collection of current state of servers or use Kafka to collect data

use crate::{error::Error, middleware::Server};

pub async fn least_connection(_available_servers: &[Server]) -> Result<Server, Error> {
    todo!()
}
