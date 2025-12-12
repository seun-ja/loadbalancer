use std::sync::atomic::Ordering;

use crate::middleware::Server;

pub async fn _latency_tracker_worker(available_servers: Vec<Server>) {
    loop {
        _check(&available_servers);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

fn _check(servers: &[Server]) {
    for server in servers {
        if server.latency_updated() {
            server.latency_update_status(false);
            let mean_latency = _mean_latency(&server.latencies);
            server
                .mean_latency
                .store(mean_latency as u64, Ordering::Relaxed);
        }
    }
}

fn _mean_latency(latencies: &[u128]) -> u128 {
    if latencies.is_empty() {
        0
    } else {
        latencies.iter().sum::<u128>() / latencies.len() as u128
    }
}
