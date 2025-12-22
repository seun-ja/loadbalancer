#![deny(clippy::disallowed_methods)]

use std::{net::SocketAddr, str::FromStr as _};

use tracing::Level;

use crate::{
    app::App,
    config::{State, SystemConfig},
};

pub mod algorithms;
mod app;
pub mod config;
pub mod db;
pub mod error;
mod middleware;
mod route;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SystemConfig::from_env()?;

    tracing_subscriber::fmt()
        .with_max_level(Level::from_str(&config.trace_level)?)
        .pretty()
        .init();

    let state = State::new(&config).await?;

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let app = App::setup(state, listener).await?;

    app.start().await
}
