//! API error types for the Lightcone REST API client.

use std::fmt;
use thiserror::Error;

/// API-specific error type for the Lightcone REST API client.
#[derive(Debug, Error)]
pub enum ApiError {
    /// HTTP/network error from reqwest
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Resource not found (404)
    #[error("Not found: {0}")]
    NotFound(ErrorResponse),

    /// Invalid request parameters (400)
    #[error("Bad request: {0}")]
    BadRequest(ErrorResponse),

    /// Permission denied, signature mismatch (403)
    #[error("Permission denied: {0}")]
    Forbidden(ErrorResponse),

    /// Rate limited (429)
    #[error("Rate limited: {0}")]
    RateLimited(ErrorResponse),

    /// Authentication required (401)
    #[error("Unauthorized: {0}")]
    Unauthorized(ErrorResponse),

    /// Resource already exists (409)
    #[error("Conflict: {0}")]
    Conflict(ErrorResponse),

    /// Server-side error (500)
    #[error("Server error: {0}")]
    ServerError(ErrorResponse),

    /// JSON deserialization error
    #[error("Deserialization error: {0}")]
    Deserialize(String),

    /// Invalid parameter provided
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Authentication required: call set_auth_token() or use builder().auth_token()
    #[error("Authentication required: call set_auth_token() or use builder().auth_token()")]
    AuthenticationRequired,

    /// Unexpected HTTP status code
    #[error("Unexpected status {0}: {1}")]
    UnexpectedStatus(u16, ErrorResponse),
}

impl ApiError {
    /// Get the server error response if this error came from an HTTP response.
    pub fn error_response(&self) -> Option<&ErrorResponse> {
        match self {
            ApiError::NotFound(resp) => Some(resp),
            ApiError::BadRequest(resp) => Some(resp),
            ApiError::Forbidden(resp) => Some(resp),
            ApiError::RateLimited(resp) => Some(resp),
            ApiError::Unauthorized(resp) => Some(resp),
            ApiError::Conflict(resp) => Some(resp),
            ApiError::ServerError(resp) => Some(resp),
            ApiError::UnexpectedStatus(_, resp) => Some(resp),
            _ => None,
        }
    }

    /// Get the HTTP status code for this error, if applicable.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            ApiError::NotFound(_) => Some(404),
            ApiError::BadRequest(_) => Some(400),
            ApiError::Forbidden(_) => Some(403),
            ApiError::RateLimited(_) => Some(429),
            ApiError::Unauthorized(_) => Some(401),
            ApiError::Conflict(_) => Some(409),
            ApiError::ServerError(_) => Some(500),
            ApiError::UnexpectedStatus(code, _) => Some(*code),
            _ => None,
        }
    }
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
    /// Create an `ErrorResponse` from a plain text string.
    ///
    /// Useful for non-JSON error bodies or synthetic error messages.
    pub fn from_text(text: String) -> Self {
        Self {
            status: None,
            message: Some(text),
            details: None,
        }
    }

    /// Get the error message, preferring `message` over `details`.
    #[deprecated(note = "Use Display formatting instead")]
    pub fn get_message(&self) -> String {
        self.message
            .clone()
            .or_else(|| self.details.clone())
            .unwrap_or_else(|| "Unknown error".to_string())
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.message, &self.details) {
            (Some(msg), Some(details)) => write!(f, "{}: {}", msg, details),
            (Some(msg), None) => write!(f, "{}", msg),
            (None, Some(details)) => write!(f, "{}", details),
            (None, None) => write!(f, "Unknown error"),
        }
    }
}
