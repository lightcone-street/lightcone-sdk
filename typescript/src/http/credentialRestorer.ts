/**
 * Host-supplied credential restoration for the HTTP transport.
 *
 * When the backend rejects a request as unauthenticated (HTTP 401), the
 * transport can ask the host application to restore its credentials — for
 * example, a browser app refreshing an expiring auth session so the auth
 * cookie is valid again — and then replay the request once. Without this
 * hook, every consumer has to detect 401s and hand-roll refresh-and-retry at
 * each call site, or surface a logout to the user whenever a short-lived
 * token expires mid-session.
 *
 * The SDK stays credential-agnostic: it never knows *how* credentials are
 * restored, only whether a replay is worthwhile. Register an implementation
 * with `LightconeClient.setCredentialRestorer`.
 *
 * Contract: the transport calls the restorer **at most once per logical
 * request**. Resolve `true` if credentials were plausibly restored, `false`
 * if restoration failed or is impossible (the original 401 propagates to the
 * caller unchanged). Whether a successful restoration also REPLAYS the
 * request depends on the request's declared retry-safety: RetryPolicy
 * Idempotent/custom requests are replayed once; RetryPolicy.None requests
 * (mutations — orders, cancels) are **never auto-replayed** — the restoration
 * still runs so the session is healed for the caller's next attempt, but the
 * 401 propagates. Concurrent 401s share one restoration (later requests
 * await the in-flight run), and the wait is bounded by a 30-second timeout.
 * A restorer rejection counts as "not restored" — the original 401 is
 * preserved, never replaced by the callback's failure.
 *
 * When the timeout elapses the transport aborts the `signal` passed to the
 * restorer and treats the restoration as failed. Promises cannot be
 * cancelled, so a restorer that ignores the signal may still be running when
 * a LATER restoration starts — implementations doing non-idempotent work
 * (e.g. a refresh-token rotation that must not run twice, or race across
 * tabs or tasks) must honor the signal or serialize internally.
 *
 * The trigger is HTTP **401 exactly** — 403 deliberately does not trigger
 * restoration (forbidden means authenticated-but-not-allowed; refreshing
 * credentials cannot change that), so recovery depends on the backend
 * signaling auth failures as real 401s. Restoration is only consulted for
 * requests to the configured API origin (redirects are never followed on the
 * API transport), and is skipped entirely for credential-management
 * endpoints (login, logout) and for cookie-override requests — the global
 * restorer restores the built-in session only. SDK calls made from INSIDE a
 * restorer must use the no-restore variants (getWithoutCredentialRestore /
 * postWithoutCredentialRestore): a restore-enabled call there awaits its own
 * restoration and only the timeout rescues it.
 */
export type CredentialRestorer = (signal?: AbortSignal) => Promise<boolean>;
