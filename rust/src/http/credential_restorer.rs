//! Host-supplied credential restoration for the HTTP transport.
//!
//! When the backend rejects a request as unauthenticated (HTTP 401), the
//! transport can ask the host application to restore its credentials — for
//! example, a browser app refreshing an expiring auth session so the auth
//! cookie is valid again — and then replay the request once. Without this
//! hook, every consumer has to detect 401s and hand-roll refresh-and-retry at
//! each call site, or surface a logout to the user whenever a short-lived
//! token expires mid-session.
//!
//! The SDK stays credential-agnostic: it never knows *how* credentials are
//! restored, only whether a replay is worthwhile. Register an implementation
//! with [`crate::LightconeClient::set_credential_restorer`].
//!
//! The trigger is HTTP **401 exactly**. 403 deliberately does not trigger
//! restoration — forbidden means the caller is authenticated but not allowed,
//! and refreshing credentials cannot change that. Recovery therefore depends
//! on the backend continuing to signal authentication failures as 401s
//! (never 403 or a rejection envelope inside a 2xx). Restoration is also
//! consulted only for requests to the configured API origin (the API
//! transport never follows redirects itself; on wasm the BROWSER may follow
//! one, which is detected best-effort from the final URL — a CORS-blocked
//! redirect target instead surfaces as a plain network error), and is
//! skipped entirely for the
//! SDK's credential-management endpoints (login, logout), for per-request
//! cookie overrides (`*_with_cookies`), and for custom `*_with_session`
//! sessions — the global restorer restores the built-in user session only.
//!
//! Reentrancy: SDK calls made from INSIDE a restorer must use the
//! no-restore variants ([`crate::http::LightconeHttp::get_without_credential_restore`],
//! [`crate::http::LightconeHttp::post_without_credential_restore`]) — a
//! restore-enabled call there ends up awaiting its own restoration, and only
//! the 30-second restoration timeout rescues it.

use std::future::Future;
use std::pin::Pin;

/// Attempt to restore request credentials after an HTTP 401.
///
/// The transport consults the restorer **at most once per logical request**:
/// return `true` if credentials were plausibly restored, `false` if
/// restoration failed or is impossible (the original 401 propagates to the
/// caller unchanged). Whether a successful restoration also REPLAYS the
/// request depends on the request's declared retry-safety: requests with
/// `RetryPolicy::Idempotent`/`Custom` are replayed once; `RetryPolicy::None`
/// requests (mutations — orders, cancels) are **never auto-replayed** — the
/// restoration still runs so the session is healed for the caller's next
/// attempt, but the 401 propagates.
///
/// Concurrent 401s share one restoration (later requests await the in-flight
/// run and reuse its outcome), and both the wait and the restorer run are
/// bounded by a 30-second timeout, after which the original 401 propagates.
/// The timeout drops the restorer's future — true cancellation on native
/// targets, but on wasm any JS work already started behind it keeps running
/// and can overlap the next restoration.
///
/// Implementations must therefore serialize concurrent restorations
/// themselves if the underlying mechanism is non-idempotent (e.g. a
/// refresh-token rotation that must not run twice or race across tabs or
/// tasks).
///
/// # Example
///
/// ```rust,ignore
/// struct BrowserSessionRestorer;
///
/// impl CredentialRestorer for BrowserSessionRestorer {
///     fn restore_credentials(&self) -> Pin<Box<dyn Future<Output = bool> + '_>> {
///         Box::pin(async {
///             // e.g. ask the auth provider's JS SDK to refresh the session
///             refresh_session().await.is_ok()
///         })
///     }
/// }
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub trait CredentialRestorer: Send + Sync {
    /// Attempt to restore credentials; `true` means a replay is worthwhile.
    ///
    /// The future is `Send` on native targets so SDK request futures remain
    /// spawnable on multi-threaded executors even with a restorer registered.
    fn restore_credentials(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>>;
}

/// Attempt to restore request credentials after an HTTP 401.
///
/// WASM variant of the trait above: browser futures (`JsFuture`-backed) are
/// not `Send`, and WASM is single-threaded, so the `Send` bound is dropped
/// from the returned future.
#[cfg(target_arch = "wasm32")]
pub trait CredentialRestorer: Send + Sync {
    /// Attempt to restore credentials; `true` means a replay is worthwhile.
    fn restore_credentials(&self) -> Pin<Box<dyn Future<Output = bool> + '_>>;
}
