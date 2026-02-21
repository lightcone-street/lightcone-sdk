//! Generic HTTP transport — retry, auth injection, error mapping.
//!
//! No endpoint-specific logic lives here. Domain sub-clients own their
//! URL construction and retry policy selection, calling `get`/`post` directly.

use crate::error::HttpError;
use crate::http::retry::{RetryConfig, RetryPolicy};

use async_lock::RwLock;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tracing;

/// Generic HTTP transport for the Lightcone REST API.
///
/// Provides `get` and `post` with retry policies, auth token injection,
/// and structured error mapping. Domain sub-clients call these directly.
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

    pub(crate) async fn set_auth_token(&self, token: Option<String>) {
        *self.auth_token.write().await = token;
    }

    pub(crate) async fn clear_auth_token(&self) {
        *self.auth_token.write().await = None;
    }

    #[allow(dead_code)]
    pub(crate) async fn has_auth_token(&self) -> bool {
        self.auth_token.read().await.is_some()
    }

    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        retry: RetryPolicy,
    ) -> Result<T, HttpError> {
        self.request_with_retry(reqwest::Method::GET, url, None::<&()>, retry)
            .await
    }

    pub(crate) async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
        retry: RetryPolicy,
    ) -> Result<T, HttpError> {
        self.request_with_retry(reqwest::Method::POST, url, Some(body), retry)
            .await
    }

    async fn request_with_retry<T: DeserializeOwned, B: Serialize>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<&B>,
        retry: RetryPolicy,
    ) -> Result<T, HttpError> {
        let config = match &retry {
            RetryPolicy::None => {
                return self.do_request(&method, url, body).await;
            }
            RetryPolicy::Idempotent => RetryConfig::idempotent(),
            RetryPolicy::Custom(c) => c.clone(),
        };

        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            match self.do_request::<T, B>(&method, url, body).await {
                Ok(resp) => return Ok(resp),
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
                        return Err(e);
                    }
                }
            }
        }

        Err(HttpError::MaxRetriesExceeded {
            attempts: config.max_retries + 1,
            last_error: last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        })
    }

    async fn do_request<T: DeserializeOwned, B: Serialize>(
        &self,
        method: &reqwest::Method,
        url: &str,
        body: Option<&B>,
    ) -> Result<T, HttpError> {
        let mut req = self.client.request(method.clone(), url);

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
            let parsed = resp.json::<T>().await?;
            return Ok(parsed);
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
