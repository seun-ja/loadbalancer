use std::net::SocketAddr;

use axum::{Router, routing::get};
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing::Level;

use crate::{
    config::{State, SystemConfig},
    middleware::request_route,
    servers::health::status,
};

pub mod config;
pub mod middleware;
pub mod servers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let config = SystemConfig::from_env()?;

    let state = State::new(&config)?;

    let server = Router::new()
        .route("/status", get(status))
        .layer(
            CorsLayer::new()
                .allow_headers(AllowHeaders::any())
                .allow_origin(AllowOrigin::any())
                .allow_methods(AllowMethods::any()),
        )
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            request_route,
        ))
        .with_state(state.clone());

    axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], config.port)))
        .serve(server.into_make_service())
        .await?;

    Ok(())
}

// background worker checking servers health status
// Load balancing algo? ref: https://www.cloudflare.com/learning/performance/types-of-load-balancing-algorithms/
