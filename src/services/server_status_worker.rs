use crate::db::RedisClient;

/// Background worker that periodically checks the status of available servers
pub async fn server_status_worker(redis_conn: RedisClient) {
    loop {
        if let Err(failing_servers) = server_status(redis_conn.clone()).await {
            // TODO: remove them from the list of available servers
            tracing::warn!("Failing servers: {:#?}", failing_servers);
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}

async fn server_status(mut redis_conn: RedisClient) -> Result<(), Vec<String>> {
    let mut failing_servers = Vec::new();

    if let Ok(data) = redis_conn.get_all_server_url().await {
        for server in data {
            if !server.is_available().await {
                failing_servers.push(server.url.to_string());
            }
        }
    }

    if failing_servers.is_empty() {
        Ok(())
    } else {
        Err(failing_servers)
    }
}
