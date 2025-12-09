use std::str::FromStr;

use axum::response::{IntoResponse, Response};
use reqwest::{Method, Response as ReqwestResponse, StatusCode, Url};

use crate::{algorithms::Algorithm, error::Error};

/// Represents a collection of server clients for load balancing
#[derive(Clone, Debug)]
pub struct ServerClients {
    pub available_servers: Vec<Server>,
}

impl ServerClients {
    pub fn new(available_servers: Vec<Server>) -> Self {
        Self { available_servers }
    }

    /// Selects a server based on a load balancing algorithm
    pub async fn selected_server(&self, algorithm: Algorithm) -> Result<Server, Error> {
        algorithm.select_server(&self.available_servers).await
    }
}

#[derive(Clone, Debug)]
pub struct Server {
    pub url: Url,
    pub client: reqwest::Client,
}

impl Server {
    pub fn new(url: &str) -> anyhow::Result<Server> {
        Ok(Self {
            url: Url::from_str(url)?,
            client: Default::default(),
        })
    }

    /// Handles incoming requests and forwards them to the server
    pub async fn handle_request(
        &self,
        method: Method,
        route: &str,
        body: Option<serde_json::Value>,
    ) -> Result<ApiResponse, Error> {
        match method {
            Method::GET => self.get_request(route, body).await,
            Method::POST => self.post_request(route, body).await,
            _ => Err(Error::MethodNotAllowed),
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
