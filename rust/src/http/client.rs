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

const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 180;

/// Auth mode for HTTP requests.
enum AuthMode {
    /// User auth via cookie (native) or credentials (WASM).
    Cookie,
    /// Per-call raw `Cookie` header override, sent verbatim. Used for
    /// server-side cookie forwarding (e.g. SSR / server functions) where the
    /// per-request browser cookies can't propagate to the SDK's process-wide
    /// token store. Carries whatever auth cookies the browser sent (e.g.
    /// `"privy-token=…; lightcone-token=…"`). On WASM this is equivalent to
    /// `Cookie` because the browser already attaches credentials.
    CookieOverride(String),
    /// Admin auth via cookie (native) or credentials (WASM).
    AdminCookie,
}

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
    admin_token: Arc<RwLock<Option<String>>>,
}

impl LightconeHttp {
    pub fn new(base_url: &str) -> Self {
        let mut builder = Client::builder();
        #[cfg(not(target_arch = "wasm32"))]
        {
            builder = builder
                .timeout(Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS))
                .pool_max_idle_per_host(10);
        }

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: builder.build().expect("Failed to build HTTP client"),
            auth_token: Arc::new(RwLock::new(None)),
            admin_token: Arc::new(RwLock::new(None)),
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

    #[allow(dead_code)]
    pub(crate) async fn set_admin_token(&self, token: String) {
        *self.admin_token.write().await = Some(token);
    }

    pub(crate) async fn clear_admin_token(&self) {
        *self.admin_token.write().await = None;
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

    /// GET with retry. Uses cookie auth.
    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.get_with_query(url, &[], retry).await
    }

    /// GET with retry and URL-encoded query parameters. Uses cookie auth.
    pub(crate) async fn get_with_query<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::GET,
            url,
            None::<&()>,
            query,
            retry,
            AuthMode::Cookie,
        )
        .await
    }

    /// GET with retry, forwarding an explicit per-call raw `Cookie` header
    /// instead of the SDK's process-wide token store. Intended for server-side
    /// cookie forwarding (SSR / server functions), where the browser's auth
    /// cookies must be relayed to the backend verbatim.
    pub(crate) async fn get_with_cookies<T: DeserializeOwned>(
        &self,
        url: &str,
        retry: RetryPolicy,
        cookie_header: &str,
    ) -> Result<T, SdkError> {
        self.get_with_cookies_and_query(url, &[], retry, cookie_header)
            .await
    }

    /// GET with retry and URL-encoded query parameters, forwarding an explicit
    /// per-call raw `Cookie` header.
    pub(crate) async fn get_with_cookies_and_query<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
        retry: RetryPolicy,
        cookie_header: &str,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::GET,
            url,
            None::<&()>,
            query,
            retry,
            AuthMode::CookieOverride(cookie_header.to_string()),
        )
        .await
    }

    /// POST with retry. Uses cookie auth.
    pub(crate) async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::POST,
            url,
            Some(body),
            &[],
            retry,
            AuthMode::Cookie,
        )
        .await
    }

    /// POST with retry. Uses admin cookie auth.
    pub(crate) async fn admin_post<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::POST,
            url,
            Some(body),
            &[],
            retry,
            AuthMode::AdminCookie,
        )
        .await
    }

    /// POST with retry, without a request body. Uses admin cookie auth.
    pub(crate) async fn admin_post_empty<T: DeserializeOwned>(
        &self,
        url: &str,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::POST,
            url,
            None::<&()>,
            &[],
            retry,
            AuthMode::AdminCookie,
        )
        .await
    }

    /// PUT with retry. Uses admin cookie auth.
    pub(crate) async fn admin_put<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::PUT,
            url,
            Some(body),
            &[],
            retry,
            AuthMode::AdminCookie,
        )
        .await
    }

    /// GET with retry. Uses admin cookie auth.
    pub(crate) async fn admin_get<T: DeserializeOwned>(
        &self,
        url: &str,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.admin_get_with_query(url, &[], retry).await
    }

    /// GET with retry and URL-encoded query parameters. Uses admin cookie auth.
    pub(crate) async fn admin_get_with_query<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::GET,
            url,
            None::<&()>,
            query,
            retry,
            AuthMode::AdminCookie,
        )
        .await
    }

    async fn request_with_retry<T: DeserializeOwned, B: Serialize>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<&B>,
        query: &[(&str, String)],
        retry: RetryPolicy,
        auth_mode: AuthMode,
    ) -> Result<T, SdkError> {
        let config = match &retry {
            RetryPolicy::None => {
                return self
                    .send_and_parse(&method, url, body, query, &auth_mode)
                    .await;
            }
            RetryPolicy::Idempotent => RetryConfig::idempotent(),
            RetryPolicy::Custom(c) => c.clone(),
        };

        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            match self
                .send_api_response::<T, B>(
                    &method,
                    url,
                    body,
                    query,
                    &auth_mode,
                    &config.retryable_statuses,
                )
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
        query: &[(&str, String)],
        auth_mode: &AuthMode,
    ) -> Result<T, SdkError> {
        let (api_resp, request_id) = self
            .send_api_response::<T, B>(method, url, body, query, auth_mode, &[])
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

    /// Low-level HTTP request: sends request and injects auth/cookies.
    async fn send_http<B: Serialize>(
        &self,
        method: &reqwest::Method,
        url: &str,
        body: Option<&B>,
        query: &[(&str, String)],
        auth_mode: &AuthMode,
    ) -> Result<(reqwest::Response, String), HttpError> {
        let request_id = Uuid::new_v4().to_string();
        let mut req = self.client.request(method.clone(), url);
        req = req.header("x-request-id", &request_id);
        if !query.is_empty() {
            req = req.query(query);
        }

        match auth_mode {
            AuthMode::Cookie => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(token) = self.auth_token.read().await.as_ref() {
                        req = req.header("Cookie", format!("lightcone-token={}", token));
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    req = req.fetch_credentials_include();
                }
            }
            AuthMode::CookieOverride(cookie_header) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    req = req.header("Cookie", cookie_header);
                }
                // On WASM the browser is already attaching the cookies via
                // credentials mode; the per-call header is unused.
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = cookie_header;
                    req = req.fetch_credentials_include();
                }
            }
            AuthMode::AdminCookie => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(token) = self.admin_token.read().await.as_ref() {
                        req = req.header("Cookie", format!("admin_token={}", token));
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    req = req.fetch_credentials_include();
                }
            }
        }

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send().await?;
        Ok((resp, request_id))
    }

    /// Sends a Lightcone API request and returns the structured ApiResponse envelope.
    ///
    /// Retryable statuses must remain HTTP-layer errors so retry policy can make
    /// the decision before API error envelopes are converted into `ApiRejected`.
    async fn send_api_response<T: DeserializeOwned, B: Serialize>(
        &self,
        method: &reqwest::Method,
        url: &str,
        body: Option<&B>,
        query: &[(&str, String)],
        auth_mode: &AuthMode,
        retryable_statuses: &[u16],
    ) -> Result<(ApiResponse<T>, String), HttpError> {
        let (resp, request_id) = self.send_http(method, url, body, query, auth_mode).await?;
        let status = resp.status();

        if status.is_success() {
            self.capture_auth_cookies(&resp).await;

            let parsed = resp.json::<ApiResponse<T>>().await?;
            return Ok((parsed, request_id));
        }

        let status_code = status.as_u16();
        let body_text = resp.text().await.unwrap_or_default();

        if retryable_statuses.contains(&status_code) {
            return Err(Self::http_error_for_status(status_code, body_text));
        }

        if Self::should_parse_api_rejection(status_code) {
            if let Ok(ApiResponse::Rejected { details }) =
                serde_json::from_str::<ApiResponse<T>>(&body_text)
            {
                return Ok((ApiResponse::Rejected { details }, request_id));
            }
        }

        Err(Self::http_error_for_status(status_code, body_text))
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn capture_auth_cookies(&self, resp: &reqwest::Response) {
        for value in resp.headers().get_all("set-cookie").iter() {
            if let Ok(header_str) = value.to_str() {
                if let Some(token) = header_str
                    .strip_prefix("lightcone-token=")
                    .and_then(|rest| rest.split(';').next())
                {
                    if !token.is_empty() {
                        *self.auth_token.write().await = Some(token.to_string());
                    }
                }
                if let Some(token) = header_str
                    .strip_prefix("admin_token=")
                    .and_then(|rest| rest.split(';').next())
                {
                    if !token.is_empty() {
                        *self.admin_token.write().await = Some(token.to_string());
                    }
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn capture_auth_cookies(&self, _resp: &reqwest::Response) {}

    fn should_parse_api_rejection(status_code: u16) -> bool {
        matches!(status_code, 400..=499) && status_code != 429
    }

    fn http_error_for_status(status_code: u16, body_text: String) -> HttpError {
        match status_code {
            401 => HttpError::Unauthorized,
            404 => HttpError::NotFound(body_text),
            429 => HttpError::RateLimited {
                retry_after_ms: None,
            },
            400..=499 => HttpError::BadRequest(body_text),
            _ => HttpError::ServerError {
                status: status_code,
                body: body_text,
            },
        }
    }
}

impl Clone for LightconeHttp {
    fn clone(&self) -> Self {
        Self {
            base_url: self.base_url.clone(),
            client: self.client.clone(),
            auth_token: self.auth_token.clone(),
            admin_token: self.admin_token.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[derive(Clone)]
    struct TestResponse {
        status: u16,
        body: &'static str,
    }

    async fn spawn_server(responses: Vec<TestResponse>) -> (String, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let responses = Arc::new(Mutex::new(VecDeque::from(responses)));
        let attempts = Arc::new(AtomicUsize::new(0));

        let server_responses = Arc::clone(&responses);
        let server_attempts = Arc::clone(&attempts);
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let responses = Arc::clone(&server_responses);
                let attempts = Arc::clone(&server_attempts);

                tokio::spawn(async move {
                    let mut buffer = [0_u8; 4096];
                    let _ = socket.read(&mut buffer).await;

                    attempts.fetch_add(1, Ordering::SeqCst);
                    let response = responses.lock().unwrap().pop_front().unwrap_or(TestResponse {
                        status: 500,
                        body: r#"{"status":"error","error_details":{"reason":"unexpected extra request"}}"#,
                    });

                    let status_text = match response.status {
                        200 => "OK",
                        400 => "Bad Request",
                        404 => "Not Found",
                        429 => "Too Many Requests",
                        503 => "Service Unavailable",
                        _ => "Error",
                    };
                    let raw_response = format!(
                        "HTTP/1.1 {} {}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        response.status,
                        status_text,
                        response.body.len(),
                        response.body
                    );
                    let _ = socket.write_all(raw_response.as_bytes()).await;
                });
            }
        });

        (format!("http://{addr}"), attempts)
    }

    fn fast_retry(statuses: Vec<u16>) -> RetryPolicy {
        RetryPolicy::Custom(RetryConfig {
            max_retries: 1,
            initial_delay: Duration::from_millis(0),
            max_delay: Duration::from_millis(0),
            backoff_factor: 1.0,
            jitter: false,
            retryable_statuses: statuses,
        })
    }

    #[tokio::test]
    async fn structured_400_returns_api_rejected_details() {
        let (base_url, _) = spawn_server(vec![TestResponse {
            status: 400,
            body: r#"{"status":"error","error_details":{"reason":"invalid tif","error_code":"INVALID_TIF","error_log_id":"LCERR_400"}}"#,
        }])
        .await;
        let http = LightconeHttp::new(&base_url);

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/test"), RetryPolicy::Idempotent)
            .await
            .unwrap_err();

        match error {
            SdkError::ApiRejected(details) => {
                assert_eq!(details.reason, "invalid tif");
                assert_eq!(details.error_code.as_deref(), Some("INVALID_TIF"));
                assert_eq!(details.error_log_id.as_deref(), Some("LCERR_400"));
                assert!(details.request_id.is_some());
            }
            other => panic!("expected ApiRejected, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn structured_404_returns_api_rejected_details() {
        let (base_url, _) = spawn_server(vec![TestResponse {
            status: 404,
            body: r#"{"status":"error","error_details":{"reason":"market not found","error_code":"NOT_FOUND","error_log_id":"LCERR_404"}}"#,
        }])
        .await;
        let http = LightconeHttp::new(&base_url);

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/missing"), RetryPolicy::None)
            .await
            .unwrap_err();

        match error {
            SdkError::ApiRejected(details) => {
                assert_eq!(details.reason, "market not found");
                assert_eq!(details.error_code.as_deref(), Some("NOT_FOUND"));
                assert_eq!(details.error_log_id.as_deref(), Some("LCERR_404"));
                assert!(details.request_id.is_some());
            }
            other => panic!("expected ApiRejected, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn structured_503_retries_before_parsing_api_rejection() {
        let (base_url, attempts) = spawn_server(vec![
            TestResponse {
                status: 503,
                body: r#"{"status":"error","error_details":{"reason":"temporarily unavailable","error_code":"UNAVAILABLE","error_log_id":"LCERR_503"}}"#,
            },
            TestResponse {
                status: 200,
                body: r#"{"status":"success","body":{"ok":true}}"#,
            },
        ])
        .await;
        let http = LightconeHttp::new(&base_url);

        let body = http
            .get::<serde_json::Value>(&format!("{base_url}/retry"), fast_retry(vec![503]))
            .await
            .unwrap();

        assert_eq!(body["ok"], true);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn structured_429_remains_retryable() {
        let (base_url, attempts) = spawn_server(vec![
            TestResponse {
                status: 429,
                body: r#"{"status":"error","error_details":{"reason":"rate limited","error_code":"RATE_LIMITED"}}"#,
            },
            TestResponse {
                status: 200,
                body: r#"{"status":"success","body":{"ok":true}}"#,
            },
        ])
        .await;
        let http = LightconeHttp::new(&base_url);

        let body = http
            .get::<serde_json::Value>(&format!("{base_url}/retry"), fast_retry(vec![429]))
            .await
            .unwrap();

        assert_eq!(body["ok"], true);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn malformed_400_falls_back_to_http_error() {
        let (base_url, _) = spawn_server(vec![TestResponse {
            status: 400,
            body: "not json",
        }])
        .await;
        let http = LightconeHttp::new(&base_url);

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/bad"), RetryPolicy::Idempotent)
            .await
            .unwrap_err();

        match error {
            SdkError::Http(HttpError::BadRequest(body)) => assert_eq!(body, "not json"),
            other => panic!("expected BadRequest, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn non_success_status_with_success_envelope_is_not_success() {
        let (base_url, _) = spawn_server(vec![TestResponse {
            status: 400,
            body: r#"{"status":"success","body":{"ok":true}}"#,
        }])
        .await;
        let http = LightconeHttp::new(&base_url);

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/bad"), RetryPolicy::Idempotent)
            .await
            .unwrap_err();

        match error {
            SdkError::Http(HttpError::BadRequest(body)) => {
                assert!(body.contains(r#""status":"success""#));
            }
            other => panic!("expected BadRequest, got {other:?}"),
        }
    }
}
