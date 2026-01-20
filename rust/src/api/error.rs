//! API error types for the Lightcone REST API client.

use thiserror::Error;

/// API-specific error type for the Lightcone REST API client.
#[derive(Debug, Error)]
pub enum ApiError {
    /// HTTP/network error from reqwest
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Resource not found (404)
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid request parameters (400)
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Permission denied, signature mismatch (403)
    #[error("Permission denied: {0}")]
    Forbidden(String),

    /// Resource already exists (409)
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Server-side error (500)
    #[error("Server error: {0}")]
    ServerError(String),

    /// JSON deserialization error
    #[error("Deserialization error: {0}")]
    Deserialize(String),

    /// Invalid parameter provided
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Unexpected HTTP status code
    #[error("Unexpected status {0}: {1}")]
    UnexpectedStatus(u16, String),
}

/// Result type alias for API operations.
pub type ApiResult<T> = Result<T, ApiError>;

/// Error response format from the API.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ErrorResponse {
    /// Error status (usually "error")
    #[serde(default)]
    pub status: Option<String>,
    /// Human-readable error message
    #[serde(alias = "error")]
    pub message: Option<String>,
    /// Additional error details
    #[serde(default)]
    pub details: Option<String>,
}

impl ErrorResponse {
    /// Get the error message, preferring `message` over `details`.
    pub fn get_message(&self) -> String {
        self.message
            .clone()
            .or_else(|| self.details.clone())
            .unwrap_or_else(|| "Unknown error".to_string())
    }
}
