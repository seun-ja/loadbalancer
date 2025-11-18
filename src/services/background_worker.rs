use crate::middleware::Server;

/// Background worker that periodically checks the status of available servers
pub async fn server_worker(available_servers: Vec<Server>) {
    loop {
        if let Err(failing_servers) = server_status(available_servers.clone()).await {
            // TODO: remove them from the list of available servers
            tracing::warn!("Failing servers: {:#?}", failing_servers);
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}

async fn server_status(available_servers: Vec<Server>) -> Result<(), Vec<String>> {
    let mut failing_servers = Vec::new();
    for server in available_servers {
        if !server.is_available().await {
            failing_servers.push(server.url.to_string())
        }
    }
    if failing_servers.is_empty() {
        Ok(())
    } else {
        Err(failing_servers)
    }
}
