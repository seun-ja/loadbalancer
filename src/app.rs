use axum::{Router, routing::get};
use tokio::{net::TcpListener, task::JoinHandle};
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::config::State;
use crate::middleware::request_route;
use crate::route::health::status;
use crate::services::{latency_tracker_worker, server_status_worker};

type JoinHandleWrapper = JoinHandle<Result<(), std::io::Error>>;

/// Application struct to hold main and background worker tasks
pub struct App {
    main: JoinHandleWrapper,
    server_status_background_worker: JoinHandleWrapper,
    latency_tracker_background_worker: JoinHandleWrapper,
}

impl App {
    pub async fn setup(
        state: State,
        listener: TcpListener,
    ) -> Result<App, Box<dyn std::error::Error>> {
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

        Ok(Self {
            main,
            server_status_background_worker,
            latency_tracker_background_worker,
        })
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        match tokio::try_join!(
            self.main,
            self.server_status_background_worker,
            self.latency_tracker_background_worker
        ) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)?,
        }
    }
}
