//! WebSocket-specific error types for the Lightcone SDK.

use thiserror::Error;

/// WebSocket-specific errors
#[derive(Debug, Clone, Error)]
pub enum WebSocketError {
    /// Initial connection failure
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Unexpected connection close
    #[error("Connection closed unexpectedly: code {code}, reason: {reason}")]
    ConnectionClosed { code: u16, reason: String },

    /// Rate limited (close code 1008)
    #[error("Rate limited: too many connections from this IP")]
    RateLimited,

    /// JSON deserialization failure
    #[error("Failed to parse message: {0}")]
    MessageParseError(String),

    /// Detected sequence gap in book updates
    #[error("Sequence gap detected: expected {expected}, received {received}")]
    SequenceGap { expected: u64, received: u64 },

    /// Server requested resync
    #[error("Resync required for orderbook: {orderbook_id}")]
    ResyncRequired { orderbook_id: String },

    /// Subscription error from server
    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),

    /// Client ping not responded
    #[error("Ping timeout: no pong response received")]
    PingTimeout,

    /// WebSocket protocol error
    #[error("WebSocket protocol error: {0}")]
    Protocol(String),

    /// Server returned an error
    #[error("Server error: {message} (code: {code})")]
    ServerError { code: String, message: String },

    /// Not connected
    #[error("Not connected to WebSocket server")]
    NotConnected,

    /// Already connected
    #[error("Already connected to WebSocket server")]
    AlreadyConnected,

    /// Send failed
    #[error("Failed to send message: {0}")]
    SendFailed(String),

    /// Channel closed
    #[error("Internal channel closed")]
    ChannelClosed,

    /// Invalid URL
    #[error("Invalid WebSocket URL: {0}")]
    InvalidUrl(String),

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// IO error
    #[error("IO error: {0}")]
    Io(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Authentication required for user stream
    #[error("Authentication required for user stream")]
    AuthRequired,

    /// HTTP request error
    #[error("HTTP request error: {0}")]
    HttpError(String),

    /// Invalid auth token
    #[error("Invalid auth token: {0}")]
    InvalidAuthToken(String),
}

impl From<tokio_tungstenite::tungstenite::Error> for WebSocketError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        use tokio_tungstenite::tungstenite::Error;
        match err {
            Error::ConnectionClosed => WebSocketError::ConnectionClosed {
                code: 1000,
                reason: "Connection closed normally".to_string(),
            },
            Error::AlreadyClosed => WebSocketError::NotConnected,
            Error::Io(e) => WebSocketError::Io(e.to_string()),
            Error::Protocol(e) => WebSocketError::Protocol(e.to_string()),
            Error::Url(e) => WebSocketError::InvalidUrl(e.to_string()),
            Error::Http(resp) => {
                WebSocketError::ConnectionFailed(format!("HTTP error: {:?}", resp.status()))
            }
            Error::HttpFormat(e) => WebSocketError::ConnectionFailed(e.to_string()),
            other => WebSocketError::Protocol(other.to_string()),
        }
    }
}

impl From<serde_json::Error> for WebSocketError {
    fn from(err: serde_json::Error) -> Self {
        WebSocketError::MessageParseError(err.to_string())
    }
}

impl From<reqwest::Error> for WebSocketError {
    fn from(err: reqwest::Error) -> Self {
        WebSocketError::HttpError(err.to_string())
    }
}

impl From<crate::auth::AuthError> for WebSocketError {
    fn from(err: crate::auth::AuthError) -> Self {
        match err {
            crate::auth::AuthError::SystemTime(msg) => WebSocketError::Protocol(msg),
            crate::auth::AuthError::HttpError(msg) => WebSocketError::HttpError(msg),
            crate::auth::AuthError::AuthenticationFailed(msg) => {
                WebSocketError::AuthenticationFailed(msg)
            }
        }
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for WebSocketError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        WebSocketError::ChannelClosed
    }
}

/// Result type alias for WebSocket operations
pub type WsResult<T> = Result<T, WebSocketError>;
