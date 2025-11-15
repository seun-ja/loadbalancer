use std::str::FromStr;

use axum::{extract::State, http::Request, middleware::Next};
use reqwest::Url;

use crate::config::State as AppState;

#[derive(Clone, Debug)]
pub struct ServerClients {
    pub available_servers: Vec<Server>,
}

impl ServerClients {
    pub fn new(available_servers: Vec<Server>) -> Self {
        Self { available_servers }
    }
    fn choiced_server(&self) -> Server {
        // implement algorithm to select server here!
        self.available_servers[0].clone() // placeholder for now
    }
}

#[derive(Clone, Debug)]
pub struct Server {
    pub url: Url,
}

impl Server {
    pub fn new(url: &str) -> anyhow::Result<Server> {
        Ok(Self {
            url: Url::from_str(url)?,
        })
    }
}

pub async fn request_route<B>(State(state): State<AppState>, req: Request<B>, _next: Next<B>) {
    let (parts, _body) = req.into_parts();

    let choiced_server = state.available_servers.choiced_server();

    let reconstructed_request_url = reconstruct_request(
        &choiced_server.url,
        &parts.uri.to_string().trim_start_matches('/'),
    );

    dbg!(reconstructed_request_url);
}

fn reconstruct_request(url: &Url, route: &str) -> String {
    format!("{url}{route}")
}
