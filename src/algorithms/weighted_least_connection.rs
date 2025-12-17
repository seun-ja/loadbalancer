use std::collections::HashMap;

use crate::error::Error;

pub async fn weighted_least_connection(
    server_loads: HashMap<String, u32>,
    weights: HashMap<String, u32>,
) -> Result<String, Error> {
    let (url, _) = server_loads
        .into_iter()
        .min_by_key(|(key, load)| {
            let weight = weights.get(key).unwrap_or(&1);
            *load / weight
        })
        .ok_or_else(|| Error::NoServerAvailable)?;

    Ok(url)
}
