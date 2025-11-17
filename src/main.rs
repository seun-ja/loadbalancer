use std::net::SocketAddr;

use axum::{Router, routing::get};
use tokio::task::JoinHandle;
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
pub mod error;
pub mod middleware;
pub mod servers;
pub mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .pretty()
        .init();

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
        // add rate limitter middleware mechanism
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            request_route,
        ))
        .with_state(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    let main: JoinHandle<Result<(), std::io::Error>> =
        tokio::spawn(async move { axum::serve(listener, server).await });

    let app = App::new(main);

    app.start().await
}

struct App {
    main: JoinHandle<Result<(), std::io::Error>>,
    // background_worker: JoinHandle<Result<(), std::io::Error>>,
}

impl App {
    fn new(main: JoinHandle<Result<(), std::io::Error>>) -> Self {
        Self { main }
    }

    async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        match tokio::try_join!(self.main) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)?,
        }
    }
}

// background worker checking servers health status
// Load balancing algo? ref: https://www.cloudflare.com/learning/performance/types-of-load-balancing-algorithms/
