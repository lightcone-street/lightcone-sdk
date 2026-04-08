//! Generic HTTP transport — retry, auth injection, ApiResponse unwrapping.
//!
//! `get()` and `post()` return `Result<T, SdkError>` directly. They handle:
//! - `x-request-id` generation and header injection
//! - Auth token injection (cookie on native, credentials on WASM)
//! - Deserialization of the `ApiResponse<T>` wrapper
//! - Unwrapping success body or converting errors to `SdkError::ApiRejected`
//!
//! `raw_post()` bypasses all of this for non-API calls (e.g. Solana JSON-RPC).

use crate::error::{HttpError, SdkError};
use crate::http::retry::{RetryConfig, RetryPolicy};
use crate::shared::api_response::ApiResponse;

use async_lock::RwLock;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tracing;
use uuid::Uuid;

/// Generic HTTP transport for the Lightcone REST API.
///
/// Provides `get` and `post` with retry policies, auth token injection,
/// and structured error mapping. Domain sub-clients call these directly:
///
/// ```rust,ignore
/// let markets: MarketsResponse = self.client.http
///     .get(&url, RetryPolicy::Idempotent)
///     .await?;
/// ```
pub struct LightconeHttp {
    base_url: String,
    client: Client,
    auth_token: Arc<RwLock<Option<String>>>,
}

impl LightconeHttp {
    pub fn new(base_url: &str) -> Self {
        let mut builder = Client::builder();
        #[cfg(not(target_arch = "wasm32"))]
        {
            builder = builder
                .timeout(Duration::from_secs(30))
                .pool_max_idle_per_host(10);
        }

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: builder.build().expect("Failed to build HTTP client"),
            auth_token: Arc::new(RwLock::new(None)),
        }
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(crate) async fn clear_auth_token(&self) {
        *self.auth_token.write().await = None;
    }

    #[allow(dead_code)]
    pub(crate) async fn has_auth_token(&self) -> bool {
        self.auth_token.read().await.is_some()
    }

    pub(crate) fn auth_token_ref(&self) -> Arc<RwLock<Option<String>>> {
        self.auth_token.clone()
    }

    /// Raw POST to an arbitrary URL (no auth, no retry, no ApiResponse wrapping).
    /// Used for Solana JSON-RPC calls.
    pub(crate) async fn raw_post<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T, HttpError> {
        let resp = self
            .client
            .post(url)
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(HttpError::ServerError {
                status: status.as_u16(),
                body: body_text,
            });
        }

        resp.json().await.map_err(Into::into)
    }

    /// GET with retry. Deserializes `ApiResponse<T>` and returns the body directly.
    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.request_with_retry(reqwest::Method::GET, url, None::<&()>, retry)
            .await
    }

    /// POST with retry. Deserializes `ApiResponse<T>` and returns the body directly.
    pub(crate) async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.request_with_retry(reqwest::Method::POST, url, Some(body), retry)
            .await
    }

    async fn request_with_retry<T: DeserializeOwned, B: Serialize>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<&B>,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        let config = match &retry {
            RetryPolicy::None => {
                return self.send_and_parse(&method, url, body).await;
            }
            RetryPolicy::Idempotent => RetryConfig::idempotent(),
            RetryPolicy::Custom(c) => c.clone(),
        };

        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            match self
                .send_request::<ApiResponse<T>, B>(&method, url, body)
                .await
            {
                Ok((api_resp, request_id)) => {
                    return Self::parse_api_response(api_resp, request_id);
                }
                Err(e) => {
                    let should_retry = match &e {
                        HttpError::ServerError { status, .. } => {
                            config.retryable_statuses.contains(status)
                        }
                        HttpError::RateLimited { retry_after_ms } => {
                            if let Some(ms) = retry_after_ms {
                                let delay = Duration::from_millis(*ms);
                                futures_timer::Delay::new(delay).await;
                            }
                            true
                        }
                        HttpError::Timeout => true,
                        #[cfg(feature = "http")]
                        HttpError::Reqwest(re) => {
                            #[cfg(not(target_arch = "wasm32"))]
                            let retryable = re.is_connect() || re.is_timeout() || re.is_request();
                            #[cfg(target_arch = "wasm32")]
                            let retryable = re.is_timeout() || re.is_request();
                            retryable
                        }
                        _ => false,
                    };

                    if should_retry && attempt < config.max_retries {
                        let delay = config.delay_for_attempt(attempt);
                        tracing::debug!(
                            attempt = attempt + 1,
                            max = config.max_retries,
                            delay_ms = delay.as_millis() as u64,
                            "Retrying request to {}",
                            url
                        );
                        futures_timer::Delay::new(delay).await;
                        last_error = Some(e);
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }

        Err(HttpError::MaxRetriesExceeded {
            attempts: config.max_retries + 1,
            last_error: last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        }
        .into())
    }

    /// High-level request: HTTP call + ApiResponse unwrap.
    async fn send_and_parse<T: DeserializeOwned, B: Serialize>(
        &self,
        method: &reqwest::Method,
        url: &str,
        body: Option<&B>,
    ) -> Result<T, SdkError> {
        let (api_resp, request_id) = self
            .send_request::<ApiResponse<T>, B>(method, url, body)
            .await?;
        Self::parse_api_response(api_resp, request_id)
    }

    /// Unwrap `ApiResponse<T>` into `Result<T, SdkError>`, attaching request_id on error.
    fn parse_api_response<T>(api_resp: ApiResponse<T>, request_id: String) -> Result<T, SdkError> {
        match api_resp {
            ApiResponse::Success { body } => Ok(body),
            ApiResponse::Rejected { mut details, .. } => {
                details.request_id = Some(request_id);
                Err(SdkError::ApiRejected(details))
            }
        }
    }

    /// Low-level HTTP request: sends request, handles auth/cookies/errors.
    /// Returns the raw deserialized body and request_id.
    /// Used by retry logic (needs `HttpError` for retry decisions).
    async fn send_request<T: DeserializeOwned, B: Serialize>(
        &self,
        method: &reqwest::Method,
        url: &str,
        body: Option<&B>,
    ) -> Result<(T, String), HttpError> {
        let request_id = Uuid::new_v4().to_string();
        let mut req = self.client.request(method.clone(), url);
        req = req.header("x-request-id", &request_id);

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(token) = self.auth_token.read().await.as_ref() {
                req = req.header("Cookie", format!("auth_token={}", token));
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            req = req.fetch_credentials_include();
        }

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send().await?;
        let status = resp.status();

        if status.is_success() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                for value in resp.headers().get_all("set-cookie").iter() {
                    if let Ok(header_str) = value.to_str() {
                        if let Some(token) = header_str
                            .strip_prefix("auth_token=")
                            .and_then(|rest| rest.split(';').next())
                        {
                            if !token.is_empty() {
                                *self.auth_token.write().await = Some(token.to_string());
                            }
                        }
                    }
                }
            }

            let parsed = resp.json::<T>().await?;
            return Ok((parsed, request_id));
        }

        let status_code = status.as_u16();
        let body_text = resp.text().await.unwrap_or_default();

        match status_code {
            401 => Err(HttpError::Unauthorized),
            404 => Err(HttpError::NotFound(body_text)),
            429 => Err(HttpError::RateLimited {
                retry_after_ms: None,
            }),
            400..=499 => Err(HttpError::BadRequest(body_text)),
            _ => Err(HttpError::ServerError {
                status: status_code,
                body: body_text,
            }),
        }
    }
}

impl Clone for LightconeHttp {
    fn clone(&self) -> Self {
        Self {
            base_url: self.base_url.clone(),
            client: self.client.clone(),
            auth_token: self.auth_token.clone(),
        }
    }
}
