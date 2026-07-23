//! Unified SDK error types.

use crate::shared::ApiRejectedDetails;
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

    #[error("Program error: {0}")]
    Program(#[from] crate::program::error::SdkError),

    #[error("Missing required market context for Market deposit source: {0}")]
    MissingMarketContext(&'static str),

    #[error("Signing error: {0}")]
    Signing(String),

    #[error("User cancelled signing")]
    UserCancelled,

    #[error("Transaction {signature} failed on-chain: {error}")]
    TransactionFailed { signature: String, error: String },

    #[error(
        "Transaction {signature} expired before confirmation — it was never processed and is safe to resubmit"
    )]
    TransactionExpired { signature: String },

    #[error(
        "Timed out confirming transaction {signature} — status unknown; check the signature on-chain before resubmitting"
    )]
    ConfirmationTimeout { signature: String },

    #[error("{0}")]
    ApiRejected(ApiRejectedDetails),

    #[error("{0}")]
    Other(String),
}

impl SdkError {
    /// True when the backend rejected the request as unauthenticated (HTTP
    /// 401) — either a bare 401 ([`HttpError::Unauthorized`]) or a 401 that
    /// carried a structured rejection envelope ([`SdkError::ApiRejected`]
    /// with an `http_status` of 401). Lets callers decide whether refreshing
    /// credentials and retrying makes sense without matching on backend
    /// error strings.
    pub fn is_unauthorized(&self) -> bool {
        match self {
            SdkError::Http(HttpError::Unauthorized) => true,
            SdkError::ApiRejected(details) => details.http_status == Some(401),
            _ => false,
        }
    }
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

    /// The browser followed a redirect off the configured API origin (WASM
    /// only — native transports never follow redirects; a 3xx surfaces as a
    /// status error instead). The response is refused so the redirect target
    /// cannot impersonate the API.
    #[error("Redirected off the API origin: {0}")]
    RedirectedOffOrigin(String),

    /// NOT produced by the SDK's HTTP retry loop. On retry exhaustion the
    /// FINAL attempt's error propagates unchanged — structured rejection
    /// details, status classification (`is_unauthorized` etc.), and request
    /// id intact — because flattening it into this wrapper's `last_error`
    /// string would destroy everything callers switch on (see the
    /// retry-exhaustion tests). The variant stays public for consumers that
    /// build their own retry loops on the raw/no-restore primitives and want
    /// a conventional exhaustion error to construct.
    #[error("Max retries exceeded after {attempts} attempts: {last_error:?}")]
    MaxRetriesExceeded {
        attempts: u32,
        last_error: Option<String>,
    },
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
    Closed { code: Option<u16>, reason: String },
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
