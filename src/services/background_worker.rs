use crate::middleware::Server;

pub async fn _server_status(available_servers: Vec<Server>) -> Result<(), Vec<String>> {
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
