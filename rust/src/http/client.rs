//! Generic HTTP transport — retry, auth injection, ApiResponse unwrapping.
//!
//! `get()` and `post()` return `Result<T, SdkError>` directly. They handle:
//! - `x-request-id` generation and header injection
//! - Auth token injection (cookie on native, credentials on WASM)
//! - Deserialization of the `ApiResponse<T>` wrapper
//! - Unwrapping success body or converting errors to `SdkError::ApiRejected`
//!
//! Auth is modeled as a [`CookieSession`] — a named cookie with a shared token
//! store. The SDK's own endpoints use the built-in user session
//! (`lightcone-token`); external crates can drive additional sessions through
//! the `*_with_session` methods.
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

#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 180;

/// Cookie name for the SDK's built-in user session.
const USER_COOKIE: &str = "lightcone-token";

/// A named cookie session: one auth cookie plus its shared token store.
///
/// - **Native**: after a successful response carrying
///   `Set-Cookie: <name>=<token>`, the token is captured into the store and
///   attached as `Cookie: <name>=<token>` on subsequent requests using this
///   session.
/// - **WASM**: the browser cookie jar stores and attaches the cookie itself
///   (requests are sent with credentials included), so the token store stays
///   empty and [`CookieSession::token`] returns `None`.
///
/// Cloning shares the token store, so a session captured through one clone of
/// the client is visible to all clones.
#[derive(Clone)]
pub struct CookieSession {
    name: String,
    token: Arc<RwLock<Option<String>>>,
}

impl CookieSession {
    pub fn new(cookie_name: impl Into<String>) -> Self {
        Self {
            name: cookie_name.into(),
            token: Arc::new(RwLock::new(None)),
        }
    }

    /// The cookie name this session attaches and captures.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The captured token, if any. Always `None` on WASM (the browser owns
    /// the cookie).
    pub async fn token(&self) -> Option<String> {
        self.token.read().await.clone()
    }

    /// Seed the session with a token obtained elsewhere (e.g. persisted from
    /// a previous process).
    pub async fn set_token(&self, token: String) {
        *self.token.write().await = Some(token);
    }

    /// Drop the captured token. Subsequent requests on this session go out
    /// without a `Cookie` header.
    pub async fn clear_token(&self) {
        *self.token.write().await = None;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn token_ref(&self) -> Arc<RwLock<Option<String>>> {
        self.token.clone()
    }
}

/// Auth mode for HTTP requests.
enum AuthMode<'a> {
    /// Auth via a named cookie session (cookie on native, credentials on WASM).
    Session(&'a CookieSession),
    /// Per-call raw `Cookie` header override, sent verbatim. Used for
    /// server-side cookie forwarding (e.g. SSR / server functions) where the
    /// per-request browser cookies can't propagate to the SDK's process-wide
    /// token store. Carries whatever auth cookies the browser sent (e.g.
    /// `"privy-token=…; lightcone-token=…"`). On WASM this is equivalent to
    /// `Session` because the browser already attaches credentials.
    CookieOverride(String),
}

#[derive(Debug)]
enum ApiRequestError {
    HttpStatus {
        status: u16,
        body: String,
        request_id: String,
        retry_after_ms: Option<u64>,
    },
    Http(HttpError),
}

impl From<HttpError> for ApiRequestError {
    fn from(error: HttpError) -> Self {
        Self::Http(error)
    }
}

impl From<reqwest::Error> for ApiRequestError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error.into())
    }
}

impl ApiRequestError {
    fn retry_after_ms(&self) -> Option<u64> {
        match self {
            Self::HttpStatus { retry_after_ms, .. } => *retry_after_ms,
            Self::Http(HttpError::RateLimited { retry_after_ms }) => *retry_after_ms,
            _ => None,
        }
    }
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
    user_session: CookieSession,
}

impl LightconeHttp {
    pub fn new(base_url: &str) -> Self {
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
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
            user_session: CookieSession::new(USER_COOKIE),
        }
    }

    /// The API base URL this transport targets (no trailing slash).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The built-in user session (`lightcone-token` cookie). Populated by the
    /// SDK after a successful login. Useful for persisting the session across
    /// processes, or as the session argument to the `*_with_session` methods.
    pub fn user_session(&self) -> &CookieSession {
        &self.user_session
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn clear_auth_token(&self) {
        self.user_session.clear_token().await;
    }

    #[allow(dead_code)]
    pub(crate) async fn has_auth_token(&self) -> bool {
        self.user_session.token().await.is_some()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn auth_token_ref(&self) -> Arc<RwLock<Option<String>>> {
        self.user_session.token_ref()
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

    /// GET with retry. Uses the user session.
    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.get_with_query(url, &[], retry).await
    }

    /// GET with retry and URL-encoded query parameters. Uses the user session.
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
            AuthMode::Session(&self.user_session),
            true,
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
            true,
        )
        .await
    }

    /// POST with retry. Uses the user session.
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
            AuthMode::Session(&self.user_session),
            true,
        )
        .await
    }

    // ── Session-parameterized requests (extension surface) ─────────────────
    //
    // These let external crates build sub-clients on top of `LightconeHttp`
    // with their own named cookie sessions. Unlike the user-session methods
    // above, a non-2xx response body that parses as the standard
    // `ApiResponse` envelope is surfaced as `SdkError::ApiRejected` instead
    // of an opaque HTTP error.

    /// GET with retry and URL-encoded query parameters, authenticated with
    /// the given [`CookieSession`].
    pub async fn get_with_session<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
        retry: RetryPolicy,
        session: &CookieSession,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::GET,
            url,
            None::<&()>,
            query,
            retry,
            AuthMode::Session(session),
            true,
        )
        .await
    }

    /// POST with retry, authenticated with the given [`CookieSession`].
    /// Pass `None::<&()>` as the body for an empty POST.
    pub async fn post_with_session<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: Option<&B>,
        retry: RetryPolicy,
        session: &CookieSession,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::POST,
            url,
            body,
            &[],
            retry,
            AuthMode::Session(session),
            true,
        )
        .await
    }

    /// PUT with retry, authenticated with the given [`CookieSession`].
    /// Pass `None::<&()>` as the body for an empty PUT.
    pub async fn put_with_session<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: Option<&B>,
        retry: RetryPolicy,
        session: &CookieSession,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::PUT,
            url,
            body,
            &[],
            retry,
            AuthMode::Session(session),
            true,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn request_with_retry<T: DeserializeOwned, B: Serialize>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<&B>,
        query: &[(&str, String)],
        retry: RetryPolicy,
        auth_mode: AuthMode<'_>,
        parse_rejected_error_body: bool,
    ) -> Result<T, SdkError> {
        let config = match &retry {
            RetryPolicy::None => {
                return self
                    .send_and_parse(
                        &method,
                        url,
                        body,
                        query,
                        &auth_mode,
                        parse_rejected_error_body,
                    )
                    .await;
            }
            RetryPolicy::Idempotent => RetryConfig::idempotent(),
            RetryPolicy::Custom(c) => c.clone(),
        };

        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            match self
                .send_api_request::<ApiResponse<T>, B>(&method, url, body, query, &auth_mode)
                .await
            {
                Ok((api_resp, request_id)) => {
                    return Self::parse_api_response(api_resp, request_id);
                }
                Err(e) => {
                    let should_retry =
                        Self::should_retry_request_error(&e, &config.retryable_statuses);

                    if should_retry && attempt < config.max_retries {
                        let delay = e
                            .retry_after_ms()
                            .map(Duration::from_millis)
                            .unwrap_or_else(|| config.delay_for_attempt(attempt));
                        tracing::debug!(
                            attempt = attempt + 1,
                            max = config.max_retries,
                            delay_ms = delay.as_millis() as u64,
                            "Retrying request to {}",
                            url
                        );
                        last_error = Some(Self::request_error_message(&e));
                        futures_timer::Delay::new(delay).await;
                    } else {
                        return Err(Self::request_error_to_sdk::<T>(
                            e,
                            parse_rejected_error_body,
                        ));
                    }
                }
            }
        }

        Err(HttpError::MaxRetriesExceeded {
            attempts: config.max_retries + 1,
            last_error: last_error.unwrap_or_else(|| "unknown".to_string()),
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
        auth_mode: &AuthMode<'_>,
        parse_rejected_error_body: bool,
    ) -> Result<T, SdkError> {
        let (api_resp, request_id) = self
            .send_api_request::<ApiResponse<T>, B>(method, url, body, query, auth_mode)
            .await
            .map_err(|e| Self::request_error_to_sdk::<T>(e, parse_rejected_error_body))?;
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

    /// Low-level HTTP request: sends request and captures auth cookies.
    /// Returns the raw deserialized body and request_id.
    /// Non-success HTTP statuses are returned with raw status/body so retry
    /// policy can decide before a backend rejection envelope is unwrapped.
    async fn send_api_request<T: DeserializeOwned, B: Serialize>(
        &self,
        method: &reqwest::Method,
        url: &str,
        body: Option<&B>,
        query: &[(&str, String)],
        auth_mode: &AuthMode<'_>,
    ) -> Result<(T, String), ApiRequestError> {
        let request_id = Uuid::new_v4().to_string();
        let mut req = self.client.request(method.clone(), url);
        req = req.header("x-request-id", &request_id);
        if !query.is_empty() {
            req = req.query(query);
        }

        match auth_mode {
            AuthMode::Session(session) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(token) = session.token.read().await.as_ref() {
                        req = req.header("Cookie", format!("{}={}", session.name, token));
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let _ = session;
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
        }

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send().await?;
        let status = resp.status();

        if status.is_success() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Capture a rotated/issued token for the session this request
                // ran under. `CookieOverride` carries the browser's cookies on
                // behalf of the user, so its capture target is the user session.
                let capture_session = match auth_mode {
                    AuthMode::Session(session) => *session,
                    AuthMode::CookieOverride(_) => &self.user_session,
                };
                let cookie_prefix = format!("{}=", capture_session.name);
                for value in resp.headers().get_all("set-cookie").iter() {
                    if let Ok(header_str) = value.to_str() {
                        if let Some(token) = header_str
                            .strip_prefix(&cookie_prefix)
                            .and_then(|rest| rest.split(';').next())
                        {
                            if !token.is_empty() {
                                capture_session.set_token(token.to_string()).await;
                            }
                        }
                    }
                }
            }

            let parsed = resp.json::<T>().await?;
            return Ok((parsed, request_id));
        }

        let status_code = status.as_u16();
        let retry_after_ms = Self::retry_after_ms(resp.headers());
        let body_text = resp.text().await.unwrap_or_default();

        Err(ApiRequestError::HttpStatus {
            status: status_code,
            body: body_text,
            request_id,
            retry_after_ms,
        })
    }

    fn should_retry_request_error(error: &ApiRequestError, retryable_statuses: &[u16]) -> bool {
        match error {
            ApiRequestError::HttpStatus { status, .. } => retryable_statuses.contains(status),
            ApiRequestError::Http(HttpError::ServerError { status, .. }) => {
                retryable_statuses.contains(status)
            }
            ApiRequestError::Http(HttpError::RateLimited { .. }) => {
                retryable_statuses.contains(&429)
            }
            ApiRequestError::Http(HttpError::Timeout) => true,
            #[cfg(feature = "http")]
            ApiRequestError::Http(HttpError::Reqwest(re)) => {
                #[cfg(not(target_arch = "wasm32"))]
                let retryable = re.is_connect() || re.is_timeout() || re.is_request();
                #[cfg(target_arch = "wasm32")]
                let retryable = re.is_timeout() || re.is_request();
                retryable
            }
            _ => false,
        }
    }

    fn request_error_to_sdk<T: DeserializeOwned>(
        error: ApiRequestError,
        parse_rejected_error_body: bool,
    ) -> SdkError {
        match error {
            ApiRequestError::HttpStatus {
                status,
                body,
                request_id,
                retry_after_ms,
            } => {
                if parse_rejected_error_body {
                    if let Some(error) = Self::parse_http_rejection::<T>(&body, request_id) {
                        return error;
                    }
                }
                Self::http_error_for_status(status, body, retry_after_ms).into()
            }
            ApiRequestError::Http(error) => error.into(),
        }
    }

    fn parse_http_rejection<T: DeserializeOwned>(
        body_text: &str,
        request_id: String,
    ) -> Option<SdkError> {
        match serde_json::from_str::<ApiResponse<T>>(body_text) {
            Ok(ApiResponse::Rejected { mut details }) => {
                details.request_id = Some(request_id);
                Some(SdkError::ApiRejected(details))
            }
            _ => None,
        }
    }

    fn request_error_message(error: &ApiRequestError) -> String {
        match error {
            ApiRequestError::HttpStatus { status, body, .. } => {
                format!("HTTP status {status}: {body}")
            }
            ApiRequestError::Http(error) => error.to_string(),
        }
    }

    fn retry_after_ms(headers: &reqwest::header::HeaderMap) -> Option<u64> {
        headers
            .get("retry-after-ms")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .or_else(|| {
                headers
                    .get("retry-after")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse::<f64>().ok())
                    .map(|seconds| (seconds * 1000.0).round().max(0.0) as u64)
            })
    }

    fn http_error_for_status(
        status_code: u16,
        body_text: String,
        retry_after_ms: Option<u64>,
    ) -> HttpError {
        match status_code {
            401 => HttpError::Unauthorized,
            404 => HttpError::NotFound(body_text),
            429 => HttpError::RateLimited { retry_after_ms },
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
            user_session: self.user_session.clone(),
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
                        409 => "Conflict",
                        429 => "Too Many Requests",
                        500 => "Internal Server Error",
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
    async fn structured_500_returns_api_rejected_details() {
        let (base_url, _) = spawn_server(vec![TestResponse {
            status: 500,
            body: r#"{"status":"error","error_details":{"reason":"engine failed","error_code":"ENGINE","error_log_id":"LCERR_500"}}"#,
        }])
        .await;
        let http = LightconeHttp::new(&base_url);

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/test"), RetryPolicy::Idempotent)
            .await
            .unwrap_err();

        match error {
            SdkError::ApiRejected(details) => {
                assert_eq!(details.reason, "engine failed");
                assert_eq!(details.error_code.as_deref(), Some("ENGINE"));
                assert_eq!(details.error_log_id.as_deref(), Some("LCERR_500"));
            }
            other => panic!("expected ApiRejected, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn custom_retry_policy_retries_raw_409_status() {
        let (base_url, attempts) = spawn_server(vec![
            TestResponse {
                status: 409,
                body: r#"{"status":"error","error_details":{"reason":"nonce mismatch","error_code":"NONCE_MISMATCH"}}"#,
            },
            TestResponse {
                status: 200,
                body: r#"{"status":"success","body":{"ok":true}}"#,
            },
        ])
        .await;
        let http = LightconeHttp::new(&base_url);

        let body = http
            .get::<serde_json::Value>(&format!("{base_url}/retry"), fast_retry(vec![409]))
            .await
            .unwrap();

        assert_eq!(body["ok"], true);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn custom_retry_policy_does_not_retry_429_when_excluded() {
        let (base_url, attempts) = spawn_server(vec![
            TestResponse {
                status: 429,
                body: r#"{"status":"error","error_details":{"reason":"rate limited","error_code":"RATE_LIMITED","error_log_id":"LCERR_429"}}"#,
            },
            TestResponse {
                status: 200,
                body: r#"{"status":"success","body":{"ok":true}}"#,
            },
        ])
        .await;
        let http = LightconeHttp::new(&base_url);

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/retry"), fast_retry(vec![503]))
            .await
            .unwrap_err();

        match error {
            SdkError::ApiRejected(details) => {
                assert_eq!(details.reason, "rate limited");
                assert_eq!(details.error_code.as_deref(), Some("RATE_LIMITED"));
                assert_eq!(details.error_log_id.as_deref(), Some("LCERR_429"));
            }
            other => panic!("expected ApiRejected, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retry_exhaustion_preserves_structured_503_details() {
        let (base_url, attempts) = spawn_server(vec![
            TestResponse {
                status: 503,
                body: r#"{"status":"error","error_details":{"reason":"temporarily unavailable","error_code":"UNAVAILABLE","error_log_id":"LCERR_503A"}}"#,
            },
            TestResponse {
                status: 503,
                body: r#"{"status":"error","error_details":{"reason":"still unavailable","error_code":"UNAVAILABLE","error_log_id":"LCERR_503B"}}"#,
            },
        ])
        .await;
        let http = LightconeHttp::new(&base_url);

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/retry"), fast_retry(vec![503]))
            .await
            .unwrap_err();

        match error {
            SdkError::ApiRejected(details) => {
                assert_eq!(details.reason, "still unavailable");
                assert_eq!(details.error_log_id.as_deref(), Some("LCERR_503B"));
            }
            other => panic!("expected ApiRejected, got {other:?}"),
        }
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
