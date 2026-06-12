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
            false,
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
            false,
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
            false,
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
                .send_request::<ApiResponse<T>, B>(
                    &method,
                    url,
                    body,
                    query,
                    &auth_mode,
                    parse_rejected_error_body,
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
        auth_mode: &AuthMode<'_>,
        parse_rejected_error_body: bool,
    ) -> Result<T, SdkError> {
        let (api_resp, request_id) = self
            .send_request::<ApiResponse<T>, B>(
                method,
                url,
                body,
                query,
                auth_mode,
                parse_rejected_error_body,
            )
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
        query: &[(&str, String)],
        auth_mode: &AuthMode<'_>,
        parse_rejected_error_body: bool,
    ) -> Result<(T, String), HttpError> {
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
        let body_text = resp.text().await.unwrap_or_default();

        // Some endpoints return the standard `ApiResponse` envelope on error
        // statuses. When requested, surface those as a parsed response (the
        // caller maps `Rejected` to `SdkError::ApiRejected`) instead of an
        // opaque HTTP error. 429 is excluded so rate limits keep retrying.
        if parse_rejected_error_body && status_code != 429 {
            if let Ok(parsed) = serde_json::from_str::<T>(&body_text) {
                return Ok((parsed, request_id));
            }
        }

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
            user_session: self.user_session.clone(),
        }
    }
}
