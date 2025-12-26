use std::collections::HashMap;

use crate::error::Error;

pub async fn least_connection(server_loads: HashMap<String, u32>) -> Result<String, Error> {
    let (url, _) = server_loads
        .into_iter()
        .min_by_key(|(_, load)| *load)
        .ok_or_else(|| Error::NoServerAvailable)?;

    Ok(url)
}
