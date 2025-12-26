use std::collections::HashMap;

use crate::error::Error;

pub async fn weighted_response_time(
    latencies: HashMap<String, u32>,
    weights: HashMap<String, u32>,
) -> Result<String, Error> {
    let (url, _) = latencies
        .into_iter()
        .min_by_key(|(key, latency)| {
            let weight = weights.get(key).unwrap_or(&1);
            *latency / weight
        })
        .ok_or_else(|| Error::NoServerAvailable)?;

    Ok(url)
}
