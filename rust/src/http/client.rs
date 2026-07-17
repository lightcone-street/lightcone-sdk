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
use crate::http::credential_restorer::CredentialRestorer;
use crate::http::retry::{RetryConfig, RetryPolicy};
use crate::shared::api_response::ApiResponse;

use async_lock::RwLock;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
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
    NonSuccessStatus {
        status: u16,
        body: String,
        request_id: String,
        headers_retry_after_ms: Option<u64>,
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
    fn headers_retry_after_ms(&self) -> Option<u64> {
        match self {
            Self::NonSuccessStatus {
                headers_retry_after_ms,
                ..
            } => *headers_retry_after_ms,
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
    /// Client for API requests. On native it never follows redirects: the API
    /// never legitimately redirects, and following one would let a redirect
    /// target observe the request (and, before this guard, trigger credential
    /// restoration while being classified under the original same-origin
    /// URL). On WASM the browser controls redirects, but it also scopes
    /// cookies per host, which is the equivalent guard there.
    api_client: Client,
    /// Client for non-API calls (`raw_post`, e.g. Solana JSON-RPC) — keeps
    /// default redirect behavior, carries no credentials.
    client: Client,
    user_session: CookieSession,
    /// Host-supplied hook consulted on HTTP 401: restore credentials (e.g.
    /// refresh an auth session). Shared across clones, like the session token
    /// store, so registering it on one clone of the client covers all of them.
    credential_restorer: Arc<RwLock<Option<Arc<dyn CredentialRestorer>>>>,
    /// Bumped after every completed restoration attempt. A request captures
    /// the epoch when it starts; if the epoch moved by the time its 401 gets
    /// to restore, another request already restored in the meantime and the
    /// stored outcome is reused instead of running the restorer again.
    restoration_epoch: Arc<AtomicU64>,
    /// Serializes restorations and stores the last outcome. Concurrent 401s
    /// AWAIT the lock (sharing the in-flight restoration) rather than failing
    /// fast; acquisition and the restorer itself are both bounded by
    /// [`CREDENTIAL_RESTORE_TIMEOUT`], so a hung restorer — or a restorer's
    /// own nested restore-enabled SDK call awaiting itself — degrades to a
    /// propagated 401 instead of a deadlock.
    restoration_gate: Arc<async_lock::Mutex<bool>>,
}

/// Upper bound on waiting for (or running) a credential restoration. Also the
/// rescue path for restorer reentrancy — see [`LightconeHttp::restoration_gate`].
#[cfg(not(test))]
const CREDENTIAL_RESTORE_TIMEOUT: Duration = Duration::from_secs(30);
/// Test builds shrink the bound so hung-restorer tests complete quickly.
#[cfg(test)]
const CREDENTIAL_RESTORE_TIMEOUT: Duration = Duration::from_millis(300);

impl LightconeHttp {
    pub fn new(base_url: &str) -> Self {
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut builder = Client::builder();
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut api_builder = Client::builder();
        #[cfg(not(target_arch = "wasm32"))]
        {
            builder = builder
                .timeout(Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS))
                .pool_max_idle_per_host(10);
            api_builder = api_builder
                .timeout(Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS))
                .pool_max_idle_per_host(10)
                .redirect(reqwest::redirect::Policy::none());
        }

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_client: api_builder.build().expect("Failed to build HTTP client"),
            client: builder.build().expect("Failed to build HTTP client"),
            user_session: CookieSession::new(USER_COOKIE),
            credential_restorer: Arc::new(RwLock::new(None)),
            restoration_epoch: Arc::new(AtomicU64::new(0)),
            restoration_gate: Arc::new(async_lock::Mutex::new(false)),
        }
    }

    /// True when `url` shares the configured API origin (scheme + host +
    /// port). Session credentials are only injected — and the credential
    /// restorer only consulted — for same-origin requests: a foreign URL that
    /// answers 401 must be able to neither trigger a restoration nor receive
    /// the restored cookie on a replay. Unparseable URLs count as foreign.
    fn is_api_origin(&self, url: &str) -> bool {
        let (Ok(request_url), Ok(base_url)) = (
            reqwest::Url::parse(url),
            reqwest::Url::parse(&self.base_url),
        ) else {
            return false;
        };
        request_url.scheme() == base_url.scheme()
            && request_url.host_str() == base_url.host_str()
            && request_url.port_or_known_default() == base_url.port_or_known_default()
    }

    /// Register (or replace) the credential restorer consulted on HTTP 401 —
    /// see [`CredentialRestorer`]. Pass through
    /// [`crate::LightconeClient::set_credential_restorer`] in normal use.
    pub async fn set_credential_restorer(&self, restorer: Arc<dyn CredentialRestorer>) {
        *self.credential_restorer.write().await = Some(restorer);
    }

    /// Run — or share — a credential restoration for a logical request that
    /// began at `start_epoch`. Returns whether replaying is worthwhile.
    ///
    /// Waiting on the gate IS awaiting an in-flight restoration, so
    /// concurrent 401s share one restorer run and one outcome. Both the wait
    /// and the restorer run are bounded by [`CREDENTIAL_RESTORE_TIMEOUT`]:
    /// a hung restorer, or a restorer whose own restore-enabled SDK call ends
    /// up awaiting itself, degrades to a propagated 401 instead of hanging.
    ///
    /// The timeout DROPS the restorer future. On native targets that is true
    /// cancellation — a timed-out restoration cannot keep running alongside
    /// the next one. On wasm, dropping the future does not stop already
    /// in-flight JS work behind it, so restorers doing non-idempotent work
    /// (e.g. rotating a refresh token) must serialize internally.
    async fn restore_credentials_shared(&self, start_epoch: u64, url: &str) -> bool {
        use futures_util::future::{select, Either};

        let lock = self.restoration_gate.lock();
        futures_util::pin_mut!(lock);
        let wait_bound = futures_timer::Delay::new(CREDENTIAL_RESTORE_TIMEOUT);
        futures_util::pin_mut!(wait_bound);
        let mut gate = match select(lock, wait_bound).await {
            Either::Left((guard, _)) => guard,
            Either::Right(_) => {
                tracing::warn!(
                    "Timed out waiting for an in-flight credential restoration; propagating 401 for {}",
                    url
                );
                return false;
            }
        };

        if self.restoration_epoch.load(Ordering::SeqCst) != start_epoch {
            // Another request completed a restoration after this one began —
            // its outcome applies to our credentials too.
            return *gate;
        }

        let restorer = self.credential_restorer.read().await.clone();
        let Some(restorer) = restorer else {
            return false;
        };

        let restore = restorer.restore_credentials();
        futures_util::pin_mut!(restore);
        let run_bound = futures_timer::Delay::new(CREDENTIAL_RESTORE_TIMEOUT);
        futures_util::pin_mut!(run_bound);
        let outcome = match select(restore, run_bound).await {
            Either::Left((restored, _)) => restored,
            Either::Right(_) => {
                tracing::warn!("Credential restorer timed out; propagating 401 for {}", url);
                false
            }
        };

        *gate = outcome;
        self.restoration_epoch.fetch_add(1, Ordering::SeqCst);
        outcome
    }

    /// Remove the credential restorer; 401s propagate to callers unchanged.
    pub async fn clear_credential_restorer(&self) {
        *self.credential_restorer.write().await = None;
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
            // Forwarded per-user cookies are outside the global credential
            // machinery: the process-wide restorer can't mint a new cookie
            // for THIS user, and a replay would resend the same stale header.
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
            true,
        )
        .await
    }

    /// POST with retry, forwarding an explicit per-call raw `Cookie` header.
    pub(crate) async fn post_with_cookies<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
        retry: RetryPolicy,
        cookie_header: &str,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::POST,
            url,
            Some(body),
            &[],
            retry,
            AuthMode::CookieOverride(cookie_header.to_string()),
            false,
        )
        .await
    }

    /// DELETE with retry. Uses the user session.
    pub(crate) async fn delete<T: DeserializeOwned>(
        &self,
        url: &str,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::DELETE,
            url,
            None::<&()>,
            &[],
            retry,
            AuthMode::Session(&self.user_session),
            true,
        )
        .await
    }

    /// DELETE with retry, forwarding an explicit per-call raw `Cookie` header.
    pub(crate) async fn delete_with_cookies<T: DeserializeOwned>(
        &self,
        url: &str,
        retry: RetryPolicy,
        cookie_header: &str,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::DELETE,
            url,
            None::<&()>,
            &[],
            retry,
            AuthMode::CookieOverride(cookie_header.to_string()),
            false,
        )
        .await
    }

    /// GET with retry, with the 401 credential restoration disabled. Use this
    /// (and its POST sibling) inside [`CredentialRestorer`] implementations
    /// for any SDK calls the restorer itself makes: a restore-enabled call
    /// from inside a restorer ends up awaiting its own restoration and only
    /// the [`CREDENTIAL_RESTORE_TIMEOUT`] rescues it.
    pub async fn get_without_credential_restore<T: DeserializeOwned>(
        &self,
        url: &str,
        retry: RetryPolicy,
    ) -> Result<T, SdkError> {
        self.request_with_retry(
            reqwest::Method::GET,
            url,
            None::<&()>,
            &[],
            retry,
            AuthMode::Session(&self.user_session),
            false,
        )
        .await
    }

    /// POST with retry, with the 401 credential restoration disabled.
    /// For credential-management endpoints (login, logout): they are the
    /// machinery restoration would re-run, so replaying them after restoring
    /// credentials is at best a no-op and at worst re-consumes single-use
    /// state (the login nonce is consumed server-side before the signature is
    /// verified, so a replayed login deterministically fails). Also for SDK
    /// calls made from inside a [`CredentialRestorer`] — see
    /// [`Self::get_without_credential_restore`].
    pub async fn post_without_credential_restore<T: DeserializeOwned, B: Serialize>(
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
            // Custom sessions are outside the global credential machinery —
            // the registered restorer restores the USER session, not these.
            false,
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
            // Custom sessions are outside the global credential machinery —
            // the registered restorer restores the USER session, not these.
            false,
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
            // Custom sessions are outside the global credential machinery —
            // the registered restorer restores the USER session, not these.
            false,
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
        allow_credential_restore: bool,
    ) -> Result<T, SdkError> {
        // `None` still means "no transport retries"; it runs through the same
        // loop with zero retry attempts so the credential-restore path below
        // covers every request. It ALSO means "never auto-replay": mutations
        // declare themselves non-idempotent via `RetryPolicy::None`, so a 401
        // still triggers restoration (healing the session for the caller's
        // next attempt) but the 401 propagates instead of replaying.
        let replay_allowed = !matches!(retry, RetryPolicy::None);
        let config = match &retry {
            RetryPolicy::None => RetryConfig {
                max_retries: 0,
                retryable_statuses: vec![],
                ..RetryConfig::default()
            },
            RetryPolicy::Idempotent => RetryConfig::idempotent(),
            RetryPolicy::Custom(c) => c.clone(),
        };

        // One request id and one body serialization per LOGICAL request:
        // transport retries and the auth replay resend the same id (tracing
        // correlation, and the hook for future server-side idempotency) and
        // the same bytes (a mutable body can't drift between attempts).
        let request_id = Uuid::new_v4().to_string();
        let body_bytes =
            match body {
                Some(b) => Some(serde_json::to_vec(b).map_err(|e| {
                    SdkError::Other(format!("Failed to serialize request body: {e}"))
                })?),
                None => None,
            };

        let start_epoch = self.restoration_epoch.load(Ordering::SeqCst);
        let mut credentials_restored = false;
        let mut attempt: u32 = 0;

        loop {
            match self
                .send_api_request::<ApiResponse<T>>(
                    &method,
                    url,
                    body_bytes.as_deref(),
                    query,
                    &auth_mode,
                    &request_id,
                )
                .await
            {
                Ok(api_resp) => {
                    return Self::parse_api_response(api_resp, request_id);
                }
                Err(e) => {
                    // On the first 401, give the host a chance to restore its
                    // credentials (e.g. refresh an auth session) — at most
                    // once per logical request, shared with any concurrent
                    // requests (see restore_credentials_shared), and only for
                    // requests to the API origin whose endpoint allows it
                    // (login/logout opt out). The replay itself additionally
                    // requires the request to have declared itself retry-safe.
                    if !credentials_restored
                        && allow_credential_restore
                        && Self::is_auth_failure(&e)
                        && self.is_api_origin(url)
                    {
                        credentials_restored = true;
                        let restored = self.restore_credentials_shared(start_epoch, url).await;
                        if restored && replay_allowed {
                            tracing::debug!("Credentials restored; replaying request to {}", url);
                            continue;
                        }
                    }

                    let should_retry =
                        Self::should_retry_request_error(&e, &config.retryable_statuses);

                    if should_retry && attempt < config.max_retries {
                        let delay = e
                            .headers_retry_after_ms()
                            .map(Duration::from_millis)
                            .unwrap_or_else(|| config.delay_for_attempt(attempt));
                        tracing::debug!(
                            attempt = attempt + 1,
                            max = config.max_retries,
                            delay_ms = delay.as_millis() as u64,
                            "Retrying request to {}",
                            url
                        );
                        attempt += 1;
                        futures_timer::Delay::new(delay).await;
                    } else {
                        return Err(Self::request_error_to_sdk::<T>(e));
                    }
                }
            }
        }
    }

    /// True for responses the backend rejected as unauthenticated (HTTP 401) —
    /// the trigger for [`CredentialRestorer`]-driven replay. Keys on the raw
    /// status, before rejection-envelope parsing, so bare and enveloped 401s
    /// are treated alike.
    fn is_auth_failure(error: &ApiRequestError) -> bool {
        matches!(error, ApiRequestError::NonSuccessStatus { status: 401, .. })
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

    /// Low-level HTTP request: sends one attempt and captures auth cookies.
    /// Non-success HTTP statuses are returned with raw status/body so retry
    /// policy can decide before a backend rejection envelope is unwrapped.
    /// The request id and pre-serialized body come from the logical request
    /// (`request_with_retry`) so every attempt sends identical bytes.
    async fn send_api_request<T: DeserializeOwned>(
        &self,
        method: &reqwest::Method,
        url: &str,
        body: Option<&[u8]>,
        query: &[(&str, String)],
        auth_mode: &AuthMode<'_>,
        request_id: &str,
    ) -> Result<T, ApiRequestError> {
        let mut req = self.api_client.request(method.clone(), url);
        req = req.header("x-request-id", request_id);
        if !query.is_empty() {
            req = req.query(query);
        }

        // Cookie injection is origin-gated: session credentials only ride to
        // the configured API origin, never to an arbitrary absolute URL a
        // caller (or a redirect target) supplies. On WASM the browser owns
        // cookie scoping — the API cookie can never reach a foreign origin —
        // but credentials mode gets the same gate so the SDK doesn't ask the
        // browser to attach ANY cookies to a URL it shouldn't be talking to.
        match auth_mode {
            AuthMode::Session(session) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if self.is_api_origin(url) {
                        if let Some(token) = session.token.read().await.as_ref() {
                            req = req.header("Cookie", format!("{}={}", session.name, token));
                        }
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let _ = session;
                    // Explicit omit on the foreign branch: the fetch default
                    // is same-origin, which would still attach PAGE-origin
                    // cookies to a URL foreign to the API.
                    if self.is_api_origin(url) {
                        req = req.fetch_credentials_include();
                    } else {
                        req = req.fetch_credentials_omit();
                    }
                }
            }
            AuthMode::CookieOverride(cookie_header) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if self.is_api_origin(url) {
                        req = req.header("Cookie", cookie_header);
                    }
                }
                // On WASM the browser is already attaching the cookies via
                // credentials mode; the per-call header is unused.
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = cookie_header;
                    if self.is_api_origin(url) {
                        req = req.fetch_credentials_include();
                    } else {
                        req = req.fetch_credentials_omit();
                    }
                }
            }
        }

        if let Some(bytes) = body {
            req = req
                .header("content-type", "application/json")
                .body(bytes.to_vec());
        }

        let resp = req.send().await?;

        // On WASM the browser follows redirects itself — reqwest exposes no
        // redirect policy there, unlike the native no-redirect `api_client` —
        // so a followed redirect is only detectable after the fact, from the
        // final URL. Refuse such responses before any status handling: the
        // request already reached the foreign target (browser cookie scoping
        // is what kept the API cookie off it), but the target must not get to
        // impersonate the API — its status would otherwise drive cookie
        // capture, retry classification, and credential restoration below.
        // Best-effort only: a redirect target that fails CORS makes send()
        // reject before the final URL is readable, surfacing as a plain
        // (retryable) network error instead of RedirectedOffOrigin.
        #[cfg(target_arch = "wasm32")]
        {
            if !self.is_api_origin(resp.url().as_str()) {
                return Err(ApiRequestError::Http(HttpError::RedirectedOffOrigin(
                    format!("{} answered from {}", url, resp.url()),
                )));
            }
        }

        let status = resp.status();

        if status.is_success() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Capture a rotated/issued token — but ONLY for the session
                // the request actually ran under. `CookieOverride` requests
                // carry one specific user's forwarded cookies; capturing
                // their Set-Cookie into the process-wide user session would
                // leak that user's rotated token to every later request from
                // a shared server client (the python SDK has always skipped
                // this; rust and typescript now match).
                if let AuthMode::Session(capture_session) = auth_mode {
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
            }

            let parsed = resp.json::<T>().await?;
            return Ok(parsed);
        }

        let status_code = status.as_u16();
        let headers_retry_after_ms = Self::retry_after_ms(resp.headers());
        let body_text = resp.text().await.unwrap_or_default();

        Err(ApiRequestError::NonSuccessStatus {
            status: status_code,
            body: body_text,
            request_id: request_id.to_string(),
            headers_retry_after_ms,
        })
    }

    fn should_retry_request_error(error: &ApiRequestError, retryable_statuses: &[u16]) -> bool {
        match error {
            ApiRequestError::NonSuccessStatus { status, .. } => retryable_statuses.contains(status),
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

    fn request_error_to_sdk<T: DeserializeOwned>(error: ApiRequestError) -> SdkError {
        match error {
            ApiRequestError::NonSuccessStatus {
                status,
                body,
                request_id,
                headers_retry_after_ms,
            } => {
                if let Some(error) = Self::parse_http_rejection::<T>(&body, request_id, status) {
                    return error;
                }
                Self::http_error_for_status(status, body, headers_retry_after_ms).into()
            }
            ApiRequestError::Http(error) => error.into(),
        }
    }

    fn parse_http_rejection<T: DeserializeOwned>(
        body_text: &str,
        request_id: String,
        http_status: u16,
    ) -> Option<SdkError> {
        match serde_json::from_str::<ApiResponse<T>>(body_text) {
            Ok(ApiResponse::Rejected { mut details }) => {
                details.request_id = Some(request_id);
                details.http_status = Some(http_status);
                Some(SdkError::ApiRejected(details))
            }
            _ => None,
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
            api_client: self.api_client.clone(),
            client: self.client.clone(),
            user_session: self.user_session.clone(),
            credential_restorer: self.credential_restorer.clone(),
            restoration_epoch: self.restoration_epoch.clone(),
            restoration_gate: self.restoration_gate.clone(),
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
    async fn cookie_forwarded_post_and_delete_retry_expected_requests() -> Result<(), SdkError> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|error| SdkError::Validation(error.to_string()))?;
        let addr = listener
            .local_addr()
            .map_err(|error| SdkError::Validation(error.to_string()))?;
        let captured = Arc::new(Mutex::new(Vec::new()));
        let server_captured = Arc::clone(&captured);
        tokio::spawn(async move {
            for request_index in 0..4 {
                let Ok((mut socket, _)) = listener.accept().await else {
                    return;
                };
                let mut buffer = [0_u8; 4096];
                let Ok(bytes_read) = socket.read(&mut buffer).await else {
                    return;
                };
                if let Ok(request) = String::from_utf8(buffer[..bytes_read].to_vec()) {
                    if let Ok(mut requests) = server_captured.lock() {
                        requests.push(request);
                    }
                }
                let (status, body) = if request_index % 2 == 0 {
                    (
                        "503 Service Unavailable",
                        r#"{"status":"error","error_details":{"reason":"retry"}}"#,
                    )
                } else {
                    ("200 OK", r#"{"status":"success","body":{"ok":true}}"#)
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(), body,
                );
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });
        let base_url = format!("http://{addr}");
        let http = LightconeHttp::new(&base_url);

        let _: serde_json::Value = http
            .post_with_cookies(
                &format!("{base_url}/favorite"),
                &serde_json::json!({}),
                RetryPolicy::Idempotent,
                "lightcone-token=test",
            )
            .await?;
        let _: serde_json::Value = http
            .delete_with_cookies(
                &format!("{base_url}/favorite"),
                RetryPolicy::Idempotent,
                "lightcone-token=test",
            )
            .await?;

        let requests = captured
            .lock()
            .map_err(|error| SdkError::Validation(error.to_string()))?;
        assert_eq!(requests.len(), 4);
        assert!(requests[0].starts_with("POST /favorite HTTP/1.1"));
        assert!(requests[1].starts_with("POST /favorite HTTP/1.1"));
        assert!(requests[2].starts_with("DELETE /favorite HTTP/1.1"));
        assert!(requests[3].starts_with("DELETE /favorite HTTP/1.1"));
        assert!(requests.iter().all(|request| request
            .to_lowercase()
            .contains("cookie: lightcone-token=test")));
        Ok(())
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

    // ── Credential restorer (401 → restore → replay) ───────────────────────

    struct StubRestorer {
        restored: bool,
        calls: Arc<AtomicUsize>,
    }

    impl CredentialRestorer for StubRestorer {
        fn restore_credentials(
            &self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let restored = self.restored;
            Box::pin(async move { restored })
        }
    }

    async fn http_with_restorer(
        base_url: &str,
        restored: bool,
    ) -> (LightconeHttp, Arc<AtomicUsize>) {
        let http = LightconeHttp::new(base_url);
        let calls = Arc::new(AtomicUsize::new(0));
        http.set_credential_restorer(Arc::new(StubRestorer {
            restored,
            calls: Arc::clone(&calls),
        }))
        .await;
        (http, calls)
    }

    #[tokio::test]
    async fn unauthorized_with_restorer_replays_once_after_restore() {
        let (base_url, attempts) = spawn_server(vec![
            TestResponse {
                status: 401,
                body: "Unauthorized",
            },
            TestResponse {
                status: 200,
                body: r#"{"status":"success","body":{"ok":true}}"#,
            },
        ])
        .await;
        let (http, restore_calls) = http_with_restorer(&base_url, true).await;

        let body = http
            .get::<serde_json::Value>(&format!("{base_url}/me"), RetryPolicy::Idempotent)
            .await
            .unwrap();

        assert_eq!(body["ok"], true);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(restore_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn no_retry_posts_restore_but_never_replay() {
        let (base_url, attempts) = spawn_server(vec![TestResponse {
            status: 401,
            body: "Unauthorized",
        }])
        .await;
        let (http, restore_calls) = http_with_restorer(&base_url, true).await;

        // RetryPolicy::None declares the request non-idempotent (orders etc.):
        // a 401 still triggers restoration — healing the session for the
        // caller's next attempt — but the request itself is NEVER auto-
        // replayed; the original 401 propagates.
        let error = http
            .post::<serde_json::Value, serde_json::Value>(
                &format!("{base_url}/order"),
                &serde_json::json!({"side": "buy"}),
                RetryPolicy::None,
            )
            .await
            .unwrap_err();

        assert!(error.is_unauthorized());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(restore_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn unauthorized_without_restorer_propagates_unchanged() {
        let (base_url, attempts) = spawn_server(vec![TestResponse {
            status: 401,
            body: "Unauthorized",
        }])
        .await;
        let http = LightconeHttp::new(&base_url);

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/me"), RetryPolicy::Idempotent)
            .await
            .unwrap_err();

        assert!(matches!(error, SdkError::Http(HttpError::Unauthorized)));
        assert!(error.is_unauthorized());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn unauthorized_with_failed_restore_propagates_without_replay() {
        let (base_url, attempts) = spawn_server(vec![TestResponse {
            status: 401,
            body: "Unauthorized",
        }])
        .await;
        let (http, restore_calls) = http_with_restorer(&base_url, false).await;

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/me"), RetryPolicy::Idempotent)
            .await
            .unwrap_err();

        assert!(error.is_unauthorized());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(restore_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn restorer_consulted_at_most_once_per_request() {
        // Restore "succeeds" but the replay still 401s (e.g. the restored
        // session is rejected too) — the second 401 must propagate rather
        // than loop through the restorer again.
        let (base_url, attempts) = spawn_server(vec![
            TestResponse {
                status: 401,
                body: "Unauthorized",
            },
            TestResponse {
                status: 401,
                body: "Unauthorized",
            },
        ])
        .await;
        let (http, restore_calls) = http_with_restorer(&base_url, true).await;

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/me"), RetryPolicy::Idempotent)
            .await
            .unwrap_err();

        assert!(error.is_unauthorized());
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(restore_calls.load(Ordering::SeqCst), 1);
    }

    /// Like `spawn_server`, but also captures each request's raw head
    /// (request line + headers) so tests can assert on what was sent.
    async fn spawn_capturing_server(
        responses: Vec<TestResponse>,
    ) -> (String, Arc<AtomicUsize>, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let responses = Arc::new(Mutex::new(VecDeque::from(responses)));
        let attempts = Arc::new(AtomicUsize::new(0));
        let captured = Arc::new(Mutex::new(Vec::new()));

        let server_responses = Arc::clone(&responses);
        let server_attempts = Arc::clone(&attempts);
        let server_captured = Arc::clone(&captured);
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let responses = Arc::clone(&server_responses);
                let attempts = Arc::clone(&server_attempts);
                let captured = Arc::clone(&server_captured);

                tokio::spawn(async move {
                    let mut buffer = [0_u8; 4096];
                    let read = socket.read(&mut buffer).await.unwrap_or(0);
                    captured
                        .lock()
                        .unwrap()
                        .push(String::from_utf8_lossy(&buffer[..read]).to_string());

                    attempts.fetch_add(1, Ordering::SeqCst);
                    let response = responses.lock().unwrap().pop_front().unwrap_or(TestResponse {
                        status: 500,
                        body: r#"{"status":"error","error_details":{"reason":"unexpected extra request"}}"#,
                    });

                    let raw_response = format!(
                        "HTTP/1.1 {} Error\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        response.status,
                        response.body.len(),
                        response.body
                    );
                    let _ = socket.write_all(raw_response.as_bytes()).await;
                });
            }
        });

        (format!("http://{addr}"), attempts, captured)
    }

    #[tokio::test]
    async fn foreign_origin_401_gets_no_cookie_and_no_restorer() {
        // The client's API origin is server A; the request goes to server B.
        // B answering 401 must neither receive the session cookie nor trigger
        // a credential restoration (a foreign endpoint could otherwise phish
        // the restored cookie via the replay).
        let (api_origin, _api_attempts) = spawn_server(vec![]).await;
        let (foreign_origin, foreign_attempts, foreign_captured) =
            spawn_capturing_server(vec![TestResponse {
                status: 401,
                body: "Unauthorized",
            }])
            .await;

        let (http, restore_calls) = http_with_restorer(&api_origin, true).await;
        http.user_session()
            .set_token("secret-token".to_string())
            .await;

        let error = http
            .get::<serde_json::Value>(&format!("{foreign_origin}/me"), RetryPolicy::Idempotent)
            .await
            .unwrap_err();

        assert!(error.is_unauthorized());
        assert_eq!(foreign_attempts.load(Ordering::SeqCst), 1);
        assert_eq!(restore_calls.load(Ordering::SeqCst), 0);
        let requests = foreign_captured.lock().unwrap();
        assert!(
            !requests[0].to_lowercase().contains("cookie:"),
            "session cookie leaked to a foreign origin: {}",
            requests[0]
        );
    }

    #[tokio::test]
    async fn same_origin_request_still_sends_cookie() {
        // Companion to the foreign-origin test: the gate must not over-block.
        let (base_url, _, captured) = spawn_capturing_server(vec![TestResponse {
            status: 200,
            body: r#"{"status":"success","body":{"ok":true}}"#,
        }])
        .await;
        let http = LightconeHttp::new(&base_url);
        http.user_session()
            .set_token("secret-token".to_string())
            .await;

        let body = http
            .get::<serde_json::Value>(&format!("{base_url}/me"), RetryPolicy::Idempotent)
            .await
            .unwrap();

        assert_eq!(body["ok"], true);
        let requests = captured.lock().unwrap();
        assert!(
            requests[0].contains("cookie: lightcone-token=secret-token")
                || requests[0].contains("Cookie: lightcone-token=secret-token"),
            "expected session cookie on a same-origin request: {}",
            requests[0]
        );
    }

    #[tokio::test]
    async fn no_restore_post_skips_restorer_on_401() {
        // Credential-management endpoints (login/logout) opt out of the
        // restore-and-replay: restoring credentials in order to log in is
        // circular, and a replayed login would re-consume single-use state.
        let (base_url, attempts) = spawn_server(vec![TestResponse {
            status: 401,
            body: "Unauthorized",
        }])
        .await;
        let (http, restore_calls) = http_with_restorer(&base_url, true).await;

        let error = http
            .post_without_credential_restore::<serde_json::Value, serde_json::Value>(
                &format!("{base_url}/api/auth/login_or_register_with_message"),
                &serde_json::json!({"message": "m"}),
                RetryPolicy::None,
            )
            .await
            .unwrap_err();

        assert!(error.is_unauthorized());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(restore_calls.load(Ordering::SeqCst), 0);
    }

    struct ReentrantRestorer {
        http: LightconeHttp,
        base_url: String,
        calls: Arc<AtomicUsize>,
    }

    impl CredentialRestorer for ReentrantRestorer {
        fn restore_credentials(
            &self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                // A restorer that calls back into the SDK must use the
                // no-restore variants: a restore-enabled call here would
                // await its own in-flight restoration and only the
                // CREDENTIAL_RESTORE_TIMEOUT would rescue it.
                let _ = self
                    .http
                    .get_without_credential_restore::<serde_json::Value>(
                        &format!("{}/nested", self.base_url),
                        RetryPolicy::None,
                    )
                    .await;
                false
            })
        }
    }

    #[tokio::test]
    async fn reentrant_restorer_terminates_and_runs_once() {
        let (base_url, attempts) = spawn_server(vec![
            TestResponse {
                status: 401,
                body: "Unauthorized",
            },
            TestResponse {
                status: 401,
                body: "Unauthorized",
            },
        ])
        .await;
        let http = LightconeHttp::new(&base_url);
        let calls = Arc::new(AtomicUsize::new(0));
        http.set_credential_restorer(Arc::new(ReentrantRestorer {
            http: http.clone(),
            base_url: base_url.clone(),
            calls: Arc::clone(&calls),
        }))
        .await;

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/me"), RetryPolicy::Idempotent)
            .await
            .unwrap_err();

        assert!(error.is_unauthorized());
        // Outer request + the restorer's nested no-restore request; no replay
        // (restore returned false) and no second restoration.
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    struct SlowRestorer {
        delay: Duration,
        calls: Arc<AtomicUsize>,
    }

    impl CredentialRestorer for SlowRestorer {
        fn restore_credentials(
            &self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let delay = self.delay;
            Box::pin(async move {
                futures_timer::Delay::new(delay).await;
                true
            })
        }
    }

    #[tokio::test]
    async fn concurrent_401s_share_one_restoration() {
        // The reviewer's probe scenario: two requests hit expiry together.
        // Both must recover, sharing a single restorer run — the second
        // request awaits the in-flight restoration instead of failing fast.
        let (base_url, attempts) = spawn_server(vec![
            TestResponse {
                status: 401,
                body: "Unauthorized",
            },
            TestResponse {
                status: 401,
                body: "Unauthorized",
            },
            TestResponse {
                status: 200,
                body: r#"{"status":"success","body":{"ok":true}}"#,
            },
            TestResponse {
                status: 200,
                body: r#"{"status":"success","body":{"ok":true}}"#,
            },
        ])
        .await;
        let http = LightconeHttp::new(&base_url);
        let calls = Arc::new(AtomicUsize::new(0));
        http.set_credential_restorer(Arc::new(SlowRestorer {
            delay: Duration::from_millis(100),
            calls: Arc::clone(&calls),
        }))
        .await;

        let first_url = format!("{base_url}/a");
        let second_url = format!("{base_url}/b");
        let (first, second) = tokio::join!(
            http.get::<serde_json::Value>(&first_url, RetryPolicy::Idempotent),
            http.get::<serde_json::Value>(&second_url, RetryPolicy::Idempotent),
        );

        assert_eq!(first.unwrap()["ok"], true);
        assert_eq!(second.unwrap()["ok"], true);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(attempts.load(Ordering::SeqCst), 4);
    }

    struct PendingRestorer {
        calls: Arc<AtomicUsize>,
    }

    impl CredentialRestorer for PendingRestorer {
        fn restore_credentials(
            &self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(std::future::pending())
        }
    }

    #[tokio::test]
    async fn hung_restorer_times_out_and_client_recovers() {
        let (base_url, attempts) = spawn_server(vec![
            TestResponse {
                status: 401,
                body: "Unauthorized",
            },
            TestResponse {
                status: 401,
                body: "Unauthorized",
            },
            TestResponse {
                status: 200,
                body: r#"{"status":"success","body":{"ok":true}}"#,
            },
        ])
        .await;
        let http = LightconeHttp::new(&base_url);
        let hung_calls = Arc::new(AtomicUsize::new(0));
        http.set_credential_restorer(Arc::new(PendingRestorer {
            calls: Arc::clone(&hung_calls),
        }))
        .await;

        // The hung restorer is abandoned at the (test-shrunk) timeout and the
        // original 401 propagates instead of hanging the request.
        let started = std::time::Instant::now();
        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/me"), RetryPolicy::Idempotent)
            .await
            .unwrap_err();
        assert!(error.is_unauthorized());
        assert!(started.elapsed() >= Duration::from_millis(250));
        assert_eq!(hung_calls.load(Ordering::SeqCst), 1);

        // The client is not stuck "restoring": a replacement restorer works.
        let (_, stub_calls) = {
            let calls = Arc::new(AtomicUsize::new(0));
            http.set_credential_restorer(Arc::new(StubRestorer {
                restored: true,
                calls: Arc::clone(&calls),
            }))
            .await;
            ((), calls)
        };
        let body = http
            .get::<serde_json::Value>(&format!("{base_url}/again"), RetryPolicy::Idempotent)
            .await
            .unwrap();
        assert_eq!(body["ok"], true);
        assert_eq!(stub_calls.load(Ordering::SeqCst), 1);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    /// Single-connection server that replies with one raw, caller-supplied
    /// HTTP response (for redirect/Set-Cookie shapes the JSON harness can't
    /// express).
    async fn spawn_raw_response_server(raw_response: String) -> (String, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let attempts = Arc::new(AtomicUsize::new(0));
        let server_attempts = Arc::clone(&attempts);
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let attempts = Arc::clone(&server_attempts);
                let raw = raw_response.clone();
                tokio::spawn(async move {
                    let mut buffer = [0_u8; 4096];
                    let _ = socket.read(&mut buffer).await;
                    attempts.fetch_add(1, Ordering::SeqCst);
                    let _ = socket.write_all(raw.as_bytes()).await;
                });
            }
        });
        (format!("http://{addr}"), attempts)
    }

    #[tokio::test]
    async fn api_redirects_are_not_followed() {
        // A redirect could bounce a same-origin request (and its eventual
        // replay) to a foreign host while origin checks still see the
        // original URL. The API transport therefore never follows redirects.
        let (foreign_url, foreign_attempts) = spawn_server(vec![]).await;
        let redirect_response = format!(
            "HTTP/1.1 302 Found\r\nlocation: {foreign_url}/steal\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
        );
        let (base_url, attempts) = spawn_raw_response_server(redirect_response).await;
        let (http, restore_calls) = http_with_restorer(&base_url, true).await;

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/me"), RetryPolicy::None)
            .await
            .unwrap_err();

        assert!(
            matches!(
                &error,
                SdkError::Http(HttpError::ServerError { status: 302, .. })
            ),
            "expected the 302 to surface as an error, got {error:?}"
        );
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(foreign_attempts.load(Ordering::SeqCst), 0);
        assert_eq!(restore_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn override_set_cookie_is_not_captured_into_user_session() {
        // A CookieOverride request carries one user's forwarded cookies;
        // its Set-Cookie must never leak into the process-wide user session
        // (cross-user contamination in shared server clients).
        let body = r#"{"status":"success","body":{"ok":true}}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\nset-cookie: lightcone-token=evil-token; Path=/\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let (base_url, _) = spawn_raw_response_server(response).await;
        let http = LightconeHttp::new(&base_url);

        let parsed: serde_json::Value = http
            .get_with_cookies(
                &format!("{base_url}/me"),
                RetryPolicy::Idempotent,
                "privy-token=forwarded",
            )
            .await
            .unwrap();

        assert_eq!(parsed["ok"], true);
        assert_eq!(http.user_session().token().await, None);
    }

    #[tokio::test]
    async fn override_401_does_not_consult_restorer() {
        let (base_url, attempts) = spawn_server(vec![TestResponse {
            status: 401,
            body: "Unauthorized",
        }])
        .await;
        let (http, restore_calls) = http_with_restorer(&base_url, true).await;

        let error = http
            .get_with_cookies::<serde_json::Value>(
                &format!("{base_url}/me"),
                RetryPolicy::Idempotent,
                "privy-token=stale",
            )
            .await
            .unwrap_err();

        assert!(error.is_unauthorized());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(restore_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn custom_session_401_does_not_consult_restorer() {
        let (base_url, attempts) = spawn_server(vec![TestResponse {
            status: 401,
            body: "Unauthorized",
        }])
        .await;
        let (http, restore_calls) = http_with_restorer(&base_url, true).await;
        let session = CookieSession::new("partner-token");

        let error = http
            .get_with_session::<serde_json::Value>(
                &format!("{base_url}/me"),
                &[],
                RetryPolicy::Idempotent,
                &session,
            )
            .await
            .unwrap_err();

        assert!(error.is_unauthorized());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(restore_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn request_id_is_stable_across_retries_and_replay() {
        let (base_url, _, captured) = spawn_capturing_server(vec![
            TestResponse {
                status: 503,
                body: "unavailable",
            },
            TestResponse {
                status: 401,
                body: "Unauthorized",
            },
            TestResponse {
                status: 200,
                body: r#"{"status":"success","body":{"ok":true}}"#,
            },
        ])
        .await;
        let (http, _) = http_with_restorer(&base_url, true).await;

        let body = http
            .get::<serde_json::Value>(&format!("{base_url}/me"), fast_retry(vec![503]))
            .await
            .unwrap();
        assert_eq!(body["ok"], true);

        let requests = captured.lock().unwrap();
        let ids: Vec<&str> = requests
            .iter()
            .map(|head| {
                head.lines()
                    .find(|line| line.to_lowercase().starts_with("x-request-id:"))
                    .map(|line| line.split_once(':').map(|(_, v)| v.trim()).unwrap_or(""))
                    .unwrap_or("")
            })
            .collect();
        assert_eq!(ids.len(), 3);
        assert!(!ids[0].is_empty());
        assert!(
            ids.iter().all(|id| *id == ids[0]),
            "transport retry and auth replay must reuse the logical request id: {ids:?}"
        );
    }

    #[tokio::test]
    async fn enveloped_401_reports_unauthorized_via_http_status() {
        let (base_url, _) = spawn_server(vec![TestResponse {
            status: 401,
            body: r#"{"status":"error","error_details":{"reason":"session expired","error_code":"SESSION_EXPIRED"}}"#,
        }])
        .await;
        let http = LightconeHttp::new(&base_url);

        let error = http
            .get::<serde_json::Value>(&format!("{base_url}/me"), RetryPolicy::Idempotent)
            .await
            .unwrap_err();

        match &error {
            SdkError::ApiRejected(details) => {
                assert_eq!(details.http_status, Some(401));
                assert_eq!(details.reason, "session expired");
            }
            other => panic!("expected ApiRejected, got {other:?}"),
        }
        assert!(error.is_unauthorized());
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
