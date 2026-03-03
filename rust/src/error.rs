//! Unified SDK error types.

use thiserror::Error;

/// Top-level SDK error.
#[derive(Error, Debug)]
pub enum SdkError {
    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),

    #[error("WebSocket error: {0}")]
    Ws(#[from] WsError),

    #[error("Auth error: {0}")]
    Auth(#[from] AuthError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

/// HTTP-layer errors.
#[derive(Error, Debug)]
pub enum HttpError {
    #[cfg(feature = "http")]
    #[error("Request failed: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Server error {status}: {body}")]
    ServerError { status: u16, body: String },

    #[error("Rate limited (retry after {retry_after_ms:?}ms)")]
    RateLimited { retry_after_ms: Option<u64> },

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Timeout")]
    Timeout,

    #[error("Max retries exceeded after {attempts} attempts: {last_error}")]
    MaxRetriesExceeded { attempts: u32, last_error: String },
}

/// WebSocket errors.
#[derive(Error, Debug)]
pub enum WsError {
    #[error("Not connected")]
    NotConnected,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Connection closed: code={code:?} reason={reason}")]
    Closed {
        code: Option<u16>,
        reason: String,
    },
}

/// Authentication errors.
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Not authenticated")]
    NotAuthenticated,

    #[error("Login failed: {0}")]
    LoginFailed(String),

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Token expired")]
    TokenExpired,
}
