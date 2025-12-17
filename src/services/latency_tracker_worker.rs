use crate::db::RedisClient;

pub async fn latency_tracker_worker(redis_conn: RedisClient) {
    loop {
        check(redis_conn.clone()).await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn check(mut redis_conn: RedisClient) {
    if let Ok(data) = redis_conn.get_servers_mean_latency_record().await {
        for (server_id, latencies) in data {
            let mean_latency = mean_latency(latencies);
            _ = redis_conn
                .update_server_mean_latency(&server_id, mean_latency) // change server_id to url
                .await;
        }
    }
}

fn mean_latency(latencies: Vec<u128>) -> u128 {
    if latencies.is_empty() {
        0
    } else {
        latencies.iter().sum::<u128>() / latencies.len() as u128
    }
}
