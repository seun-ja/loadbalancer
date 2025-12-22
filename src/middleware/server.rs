use std::str::FromStr as _;

use axum::response::{IntoResponse, Response};
use reqwest::{Method, Response as ReqwestResponse, StatusCode, Url};
use serde::{Deserialize, Serialize};

use crate::{db::RedisClient, error::Error};

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

#[derive(Serialize, Deserialize, Clone)]
pub struct StaticServerData {
    pub url: Url,
    pub weight: u32,
}

impl StaticServerData {
    pub fn from_json(data: String) -> Result<Self, Error> {
        serde_json::from_str(&data).map_err(Error::SerializationError)
    }

    pub fn new(url_and_weight: &str) -> anyhow::Result<Self> {
        let (url, weight) = url_and_weight
            .split_once('$')
            .ok_or_else(|| anyhow::anyhow!("Invalid server format, expected 'url$weight'"))?;

        let weight = weight
            .parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid weight, expected a positive integer"))?;

        let url = Url::from_str(url)?;

        Ok(Self { url, weight })
    }

    pub fn static_data(self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(&self)?)
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
