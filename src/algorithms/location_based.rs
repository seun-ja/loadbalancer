use std::{collections::HashMap, sync::LazyLock};

use crate::error::Error;

pub async fn location_based(location: &str) -> Result<String, Error> {
    SERVERS_LOCATION_MAPPINGS // Probably not the best solution, but it works for now
        .get(location)
        .cloned()
        .ok_or(Error::InternalServerError)
}

static SERVERS_LOCATION_MAPPINGS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    HashMap::from([
        // ======================
        // North America
        // ======================
        ("us".into(), "https://us-east.example.com".into()),
        ("us-east".into(), "https://us-east.example.com".into()),
        ("us-central".into(), "https://us-east.example.com".into()),
        ("ca".into(), "https://us-east.example.com".into()),
        ("ca-east".into(), "https://us-east.example.com".into()),
        ("mx".into(), "https://us-east.example.com".into()),
        ("us-west".into(), "https://us-west.example.com".into()),
        ("ca-west".into(), "https://us-west.example.com".into()),
        // ======================
        // Europe
        // ======================
        ("ie".into(), "https://eu-west.example.com".into()),
        ("uk".into(), "https://eu-west.example.com".into()),
        ("fr".into(), "https://eu-west.example.com".into()),
        ("de".into(), "https://eu-west.example.com".into()),
        ("nl".into(), "https://eu-west.example.com".into()),
        ("be".into(), "https://eu-west.example.com".into()),
        ("es".into(), "https://eu-west.example.com".into()),
        ("pt".into(), "https://eu-west.example.com".into()),
        ("pl".into(), "https://eu-central.example.com".into()),
        ("cz".into(), "https://eu-central.example.com".into()),
        ("at".into(), "https://eu-central.example.com".into()),
        ("ch".into(), "https://eu-central.example.com".into()),
        ("hu".into(), "https://eu-central.example.com".into()),
        // ======================
        // Africa
        // ======================
        ("ng".into(), "https://africa.example.com".into()),
        ("gh".into(), "https://africa.example.com".into()),
        ("ke".into(), "https://africa.example.com".into()),
        ("za".into(), "https://africa.example.com".into()),
        ("eg".into(), "https://africa.example.com".into()),
        // ======================
        // Middle East
        // ======================
        ("ae".into(), "https://middle-east.example.com".into()),
        ("sa".into(), "https://middle-east.example.com".into()),
        ("qa".into(), "https://middle-east.example.com".into()),
        ("il".into(), "https://middle-east.example.com".into()),
        // ======================
        // Asia
        // ======================
        ("in".into(), "https://asia-south.example.com".into()),
        ("pk".into(), "https://asia-south.example.com".into()),
        ("bd".into(), "https://asia-south.example.com".into()),
        ("lk".into(), "https://asia-south.example.com".into()),
        ("jp".into(), "https://asia-east.example.com".into()),
        ("kr".into(), "https://asia-east.example.com".into()),
        ("tw".into(), "https://asia-east.example.com".into()),
        ("sg".into(), "https://asia-southeast.example.com".into()),
        ("id".into(), "https://asia-southeast.example.com".into()),
        ("th".into(), "https://asia-southeast.example.com".into()),
        ("vn".into(), "https://asia-southeast.example.com".into()),
        ("ph".into(), "https://asia-southeast.example.com".into()),
        // ======================
        // Oceania
        // ======================
        ("au".into(), "https://australia.example.com".into()),
        ("nz".into(), "https://australia.example.com".into()),
        // ======================
        // Fallback / Generic
        // ======================
        ("global".into(), "https://us-east.example.com".into()),
    ])
});
