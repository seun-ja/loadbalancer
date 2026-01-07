use axum::{
    body::{BodyDataStream, Bytes},
    extract::State,
    http::Request,
    middleware::Next,
    response::IntoResponse,
};
use futures_util::stream::StreamExt;
use serde_json::Value;

use crate::{config::State as AppState, error::Error};

mod server;

pub use server::{ServerClient, StaticServerData};

/// Middleware function to route requests to appropriate servers
pub async fn request_route(
    State(mut state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<impl IntoResponse, Error> {
    if req.uri().path().starts_with("/status") {
        return Ok(next.run(req).await);
    }

    let (parts, body) = req.into_parts();

    tracing::info!("New Request Received");

    // TODO: How expensive is this?
    let json_body = BodyBytes::from_body_data_stream(body.into_data_stream())
        .await?
        .to_json()
        .ok();

    let route = parts.uri.to_string();

    let server_client = state
        .algorithm
        .select_server(state.redis_conn.clone())
        .await?;

    let start_time = std::time::Instant::now();

    let response = server_client
        .handle_request(
            parts.method,
            route.trim_start_matches('/'),
            json_body,
            state.redis_conn.clone(),
        )
        .await?;

    let latency = start_time.elapsed().as_millis();

    // TODO: move to background
    state
        .redis_conn
        .update_server_latency_record(server_client.url.as_str(), latency)
        .await?;

    Ok(response.into_response())
}

struct BodyBytes(Bytes);

impl BodyBytes {
    async fn from_body_data_stream(mut body_stream: BodyDataStream) -> anyhow::Result<Self> {
        let mut body = Vec::new();

        while let Some(chunk_result) = body_stream.next().await {
            let chunk = chunk_result?;
            body.extend_from_slice(&chunk);
        }

        Ok(BodyBytes(Bytes::from(body)))
    }

    fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_slice(&self.0)
    }
}
