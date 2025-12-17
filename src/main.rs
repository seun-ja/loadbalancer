#![deny(clippy::disallowed_methods)]

use std::{net::SocketAddr, str::FromStr as _};

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
    route::health::status,
    services::{latency_tracker_worker, server_status_worker},
};

pub mod algorithms;
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

    let main = tokio::spawn(async move { axum::serve(listener, server).await });

    let redis_conn_1 = state.redis_conn.clone();
    let redis_conn_2 = state.redis_conn.clone();

    let server_status_background_worker = tokio::spawn(async move {
        let _: () = server_status_worker(redis_conn_1).await;
        Ok(())
    });

    let latency_tracker_background_worker = tokio::spawn(async move {
        let _: () = latency_tracker_worker(redis_conn_2).await;
        Ok(())
    });

    let app = App::new(
        main,
        server_status_background_worker,
        latency_tracker_background_worker,
    );

    app.start().await
}

type JoinHandleWrapper = JoinHandle<Result<(), std::io::Error>>;

/// Application struct to hold main and background worker tasks
struct App {
    main: JoinHandleWrapper,
    background_worker: JoinHandleWrapper,
    latency_tracker_background_worker: JoinHandleWrapper,
}

impl App {
    fn new(
        main: JoinHandleWrapper,
        background_worker: JoinHandleWrapper,
        latency_tracker_background_worker: JoinHandleWrapper,
    ) -> Self {
        Self {
            main,
            background_worker,
            latency_tracker_background_worker,
        }
    }

    async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        match tokio::try_join!(
            self.main,
            self.background_worker,
            self.latency_tracker_background_worker
        ) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)?,
        }
    }
}

// background worker checking servers health status
// Load balancing algo? ref: https://www.cloudflare.com/learning/performance/types-of-load-balancing-algorithms/
