use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

/// Custom error types for the load balancer application
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not Found")]
    NotFound,
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Other: {0}")]
    Other(#[from] anyhow::Error),
    #[error("Method Not Allowed")]
    MethodNotAllowed,
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("Invalid Response")]
    InvalidResponse,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::NotFound => (StatusCode::NOT_FOUND, self).into_response(),
            Error::InternalServerError | Error::Other(_) | Error::InvalidResponse => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, self).into_response(),
            Error::MethodNotAllowed => (StatusCode::METHOD_NOT_ALLOWED, self).into_response(),
            Error::InvalidUrl => (StatusCode::BAD_REQUEST, self).into_response(),
        }
    }
}
