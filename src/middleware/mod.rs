use axum::{
    body::{BodyDataStream, Bytes},
    extract::State,
    http::Request,
    middleware::Next,
};
use futures_util::stream::StreamExt;
use serde_json::Value;

use crate::{config::State as AppState, error::Error, middleware::server::ApiResponse};

mod server;

pub use server::{Server, ServerClients};

pub async fn request_route(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    _next: Next,
) -> Result<ApiResponse, Error> {
    let (parts, body) = req.into_parts();

    tracing::info!("New Request Received");

    let body_data_stream = body.into_data_stream();
    // TODO: add mechanism to check if the required route expecting a body gets if not throw error
    let json_body = BodyBytes::from_body_data_stream(body_data_stream)
        .await?
        .to_json()
        .ok();

    let route = parts.uri.to_string();

    state
        .available_servers
        .choiced_server()
        .handle_request(parts.method, route.trim_start_matches('/'), json_body)
        .await
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
