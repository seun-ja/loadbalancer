use std::str::FromStr;

use axum::response::{IntoResponse, Response};
use reqwest::{Method, Response as ReqwestResponse, StatusCode, Url};

use crate::{
    db::{RedisClient, StaticServerData},
    error::Error,
};

#[derive(Clone)]
pub struct ServerClient {
    pub url: Url,
    pub client: reqwest::Client,
}

impl ServerClient {
    /// Handles incoming requests and forwards them to the server
    pub async fn handle_request(
        &self,
        method: Method,
        route: &str,
        body: Option<serde_json::Value>,
        mut redis_conn: RedisClient,
    ) -> Result<ApiResponse, Error> {
        let result = match method {
            Method::GET => self.get_request(route, body).await,
            Method::POST => self.post_request(route, body).await,
            _ => return Err(Error::MethodNotAllowed),
        };

        // Update load once, regardless of success or failure
        redis_conn.update_server_load(self.url.as_str(), 1).await?;

        result
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

impl From<StaticServerData> for ServerClient {
    fn from(value: StaticServerData) -> Self {
        Self {
            url: value.url,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Clone)]
pub struct Server {
    pub url: Url,
    pub client: reqwest::Client,
    load: u32,
    weight: u32,
    pub mean_latency: u128,
    pub latencies: Vec<u128>,
    latencies_updated: bool,
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
            load: 0,
            weight,
            mean_latency: 0,
            latencies: Vec::new(),
            latencies_updated: false,
        })
    }

    /// Returns the current load of the server
    pub fn load(&self) -> u32 {
        self.load
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
        self.latencies_updated
    }

    pub fn mean_latency(&self) -> u128 {
        self.mean_latency
    }

    pub fn static_data(self) -> anyhow::Result<String> {
        let static_data: StaticServerData = self.into();
        Ok(serde_json::to_string(&static_data)?)
    }
}

impl From<Server> for StaticServerData {
    fn from(info: Server) -> Self {
        Self {
            url: info.url,
            weight: info.weight,
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
