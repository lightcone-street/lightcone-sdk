"""Host-supplied credential restoration for the HTTP transport.

When the backend rejects a request as unauthenticated (HTTP 401), the
transport can ask the host application to restore its credentials — for
example, refreshing an expiring auth session so the auth cookie is valid
again — and then replay the request once. Without this hook, every consumer
has to detect 401s and hand-roll refresh-and-retry at each call site, or
surface a logout whenever a short-lived token expires mid-session.

The SDK stays credential-agnostic: it never knows *how* credentials are
restored, only whether a replay is worthwhile. Register an implementation
with ``LightconeClient.set_credential_restorer``.

Contract: the transport calls the restorer **at most once per logical
request**. Return ``True`` if credentials were plausibly restored, ``False``
if restoration failed or is impossible (the original 401 propagates to the
caller unchanged). Whether a successful restoration also REPLAYS the request
depends on its declared retry-safety: ``RetryPolicy.IDEMPOTENT``/custom
requests are replayed once; ``RetryPolicy.NONE`` requests (mutations —
orders, cancels) are **never auto-replayed** — the restoration still runs so
the session heals for the caller's next attempt, but the 401 propagates.
Concurrent 401s share one restoration (later requests await the in-flight
run), bounded by a 30-second timeout. A restorer exception counts as "not
restored" — the original 401 is preserved.

Implementations should serialize concurrent restorations themselves if the
underlying mechanism requires it (e.g. a refresh-token rotation that must
not race across tasks).

The trigger is HTTP **401 exactly** — 403 deliberately does not trigger
restoration (forbidden means authenticated-but-not-allowed; refreshing
credentials cannot change that), so recovery depends on the backend
signaling auth failures as real 401s. Restoration is only consulted for
requests to the configured API origin (redirects are never followed on the
API transport), and is skipped entirely for credential-management endpoints
(login, logout) and for ``get_with_cookies`` overrides — the global restorer
restores the built-in session only. SDK calls made from INSIDE a restorer
must pass ``allow_credential_restore=False``: a restore-enabled call there
awaits its own restoration and only the timeout rescues it.
"""

from collections.abc import Awaitable, Callable

CredentialRestorer = Callable[[], Awaitable[bool]]

__all__ = ["CredentialRestorer"]
