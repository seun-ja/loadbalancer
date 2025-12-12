use std::{
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
};

use axum::response::{IntoResponse, Response};
use reqwest::{Method, Response as ReqwestResponse, StatusCode, Url};

use crate::{algorithms::Algorithm, error::Error};

/// Represents a collection of server clients for load balancing
#[derive(Clone)]
pub struct ServerClients {
    pub available_servers: Vec<Server>,
}

impl ServerClients {
    pub fn new(available_servers: Vec<Server>) -> Self {
        Self { available_servers }
    }

    /// Selects a server based on a load balancing algorithm
    pub async fn selected_server(&self, algorithm: Algorithm) -> Result<Server, Error> {
        algorithm
            .select_server(&self.available_servers)
            .await
            .inspect(|s| {
                s.load.fetch_add(1, Ordering::Acquire);
            })
    }
}

#[derive(Clone)]
pub struct Server {
    pub url: Url,
    pub client: reqwest::Client,
    load: Arc<AtomicU32>,
    weight: u32,
    pub mean_latency: Arc<AtomicU64>,
    pub latencies: Vec<u128>,
    latencies_updated: Arc<AtomicBool>,
}

impl Server {
    pub fn new(url_and_weight: &str) -> anyhow::Result<Server> {
        let (url, weight) = url_and_weight
            .split_once('$')
            .ok_or_else(|| anyhow::anyhow!("Invalid server format, expected 'url$weight'"))?;

        let weight = weight
            .parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid weight, expected a positive integer"))?;

        let url = Url::from_str(url)?;

        Ok(Self {
            url,
            client: Default::default(),
            load: Arc::new(AtomicU32::new(0)),
            weight,
            mean_latency: Arc::new(AtomicU64::new(0)),
            latencies: Vec::new(),
            latencies_updated: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Returns the current load of the server
    pub fn load(&self) -> u32 {
        self.load.load(Ordering::Relaxed)
    }

    /// Returns the weight of the server
    pub fn weight(&self) -> u32 {
        self.weight
    }

    pub fn update_latencies(&mut self, latency: u128) {
        if self.latencies.len() >= 20 {
            // TODO: make it customisable
            self.latencies.remove(0);
        }
        self.latencies.push(latency);
    }

    pub fn latency_updated(&self) -> bool {
        self.latencies_updated.load(Ordering::Relaxed)
    }

    pub fn latency_update_status(&self, b: bool) {
        self.latencies_updated.store(b, Ordering::Relaxed)
    }

    pub fn mean_latency(&self) -> u64 {
        self.mean_latency.load(Ordering::Relaxed)
    }

    /// Handles incoming requests and forwards them to the server
    pub async fn handle_request(
        &self,
        method: Method,
        route: &str,
        body: Option<serde_json::Value>,
    ) -> Result<ApiResponse, Error> {
        // TODO: What if the request fails is the load count reduced?
        match method {
            Method::GET => self.get_request(route, body).await.inspect(|_| {
                self.load.fetch_sub(1, Ordering::Release);
            }),
            Method::POST => self.post_request(route, body).await.inspect(|_| {
                self.load.fetch_sub(1, Ordering::Release);
            }),
            _ => {
                self.load.fetch_sub(1, Ordering::Release);
                Err(Error::MethodNotAllowed)
            }
        }
    }

    /// Sends a POST request to the server
    pub async fn post_request(
        &self,
        route: &str,
        body: Option<serde_json::Value>,
    ) -> Result<ApiResponse, Error> {
        let url = self.url.join(route).map_err(|_| Error::InvalidUrl)?;

        let post = self.client.post(url);

        let response = if let Some(body) = body {
            post.body(body.to_string())
                .send()
                .await
                .map(ApiResponse::from_response)
        } else {
            post.send().await.map(ApiResponse::from_response)
        };

        response.map_err(|e| Error::Other(e.into()))?.await
    }

    /// Sends a GET request to the server
    pub async fn get_request(
        &self,
        route: &str,
        body: Option<serde_json::Value>,
    ) -> Result<ApiResponse, Error> {
        let url = self.url.join(route).map_err(|_| Error::InvalidUrl)?;

        let get = self.client.get(url);

        let response = if let Some(body) = body {
            get.body(body.to_string())
                .send()
                .await
                .map(ApiResponse::from_response)
        } else {
            get.send().await.map(ApiResponse::from_response)
        };

        response.map_err(|e| Error::Other(e.into()))?.await
    }

    /// Checks if the server is available by sending a request to the `/status` endpoint
    pub async fn is_available(&self) -> bool {
        if let Ok(url) = self.url.join("/status") {
            match self.client.get(url).send().await {
                Ok(response) => response.status().is_success(),
                Err(_) => false,
            }
        } else {
            false
        }
    }
}

pub struct ApiResponse {
    status: StatusCode,
    message: String,
}

impl ApiResponse {
    async fn from_response(response: ReqwestResponse) -> Result<Self, Error> {
        let status = response.status();
        let message = response.text().await.map_err(|_| Error::InvalidResponse)?;

        Ok(Self { status, message })
    }
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}
