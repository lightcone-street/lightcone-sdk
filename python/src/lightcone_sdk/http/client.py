"""HTTP client for the Lightcone SDK.

``get()`` and ``post()`` return the unwrapped API body directly. They handle:
- ``x-request-id`` generation and header injection
- auth cookie injection
- deserialization of the ``ApiResponse`` wrapper
- conversion of backend rejections into ``ApiRejected``

``raw_post()`` bypasses ApiResponse handling for non-API calls such as Solana
JSON-RPC.
"""

from __future__ import annotations

import asyncio
import json
import logging
import uuid
from enum import Enum
from typing import Any, Optional

import aiohttp

from ..error import ApiRejected, HttpError, HttpErrorKind
from ..shared.api_response import ApiRejectedDetails, ApiResponse
from .credential_restorer import CredentialRestorer
from .retry import RetryConfig, RetryPolicy, delay_for_attempt

logger = logging.getLogger(__name__)

DEFAULT_TIMEOUT_SECS = 180


class _HttpStatusError(Exception):
    def __init__(
        self,
        status: int,
        body: str,
        headers: aiohttp.typedefs.LooseHeaders,
        request_id: str,
    ):
        super().__init__(f"HTTP status {status}: {body}")
        self.status = status
        self.body = body
        self.headers = headers
        self.request_id = request_id


class _AuthMode(str, Enum):
    COOKIE = "cookie"
    COOKIE_OVERRIDE = "cookie_override"


class LightconeHttp:
    """HTTP client with retry, auth, and ApiResponse unwrapping."""

    def __init__(
        self,
        base_url: str,
        timeout: int = DEFAULT_TIMEOUT_SECS,
    ):
        self._base_url = base_url.rstrip("/")
        self._auth_token: Optional[str] = None
        self._timeout = aiohttp.ClientTimeout(total=timeout)
        self._session: Optional[aiohttp.ClientSession] = None
        self._credential_restorer: Optional[CredentialRestorer] = None
        # The in-flight restoration, shared by every request that 401s while
        # it runs — concurrent 401s await the same task (bounded by the
        # timeout below) instead of failing fast.
        self._restoration_task: Optional[asyncio.Task[bool]] = None
        # Set (to a loop-clock deadline) when a timeout first requests
        # cancellation of the CURRENT slot task. The slot stays occupied while
        # the task unwinds (no overlap); only a waiter timing out AFTER this
        # shared deadline hard-abandons it. A per-waiter flag instead of a
        # shared deadline would let two near-simultaneous timeouts abandon the
        # slot immediately — the second waiter would mistake the first's
        # just-requested cancellation for an ignored one.
        self._restoration_cancel_deadline: Optional[float] = None
        # Bumped after every completed restoration. A request captures the
        # epoch when it starts; if the epoch moved by the time its 401 gets to
        # restore, another request already restored and the stored outcome is
        # reused.
        self._restoration_epoch = 0
        self._last_restore_outcome = False
        # Upper bound on waiting for (or running) a restoration. Also the
        # rescue for restorer reentrancy: a restorer whose own restore-enabled
        # SDK call awaits its own restoration degrades to a propagated 401
        # after this bound instead of deadlocking. Overridable in tests.
        self._credential_restore_timeout = 30.0

    @property
    def base_url(self) -> str:
        return self._base_url

    @property
    def auth_token(self) -> Optional[str]:
        """Public accessor for the auth token."""
        return self._auth_token

    def set_auth_token(self, token: Optional[str]) -> None:
        """Set or clear the auth token."""
        self._auth_token = token

    def clear_auth_token(self) -> None:
        """Clear the auth token."""
        self._auth_token = None

    def has_auth_token(self) -> bool:
        return self._auth_token is not None

    def set_credential_restorer(self, restorer: CredentialRestorer) -> None:
        """Register (or replace) the credential restorer consulted on HTTP 401.

        See :mod:`lightcone_sdk.http.credential_restorer`. Pass through
        ``LightconeClient.set_credential_restorer`` in normal use.
        """
        self._credential_restorer = restorer

    def clear_credential_restorer(self) -> None:
        """Remove the credential restorer; 401s propagate to callers unchanged."""
        self._credential_restorer = None

    async def _restore_credentials_shared(self, start_epoch: int) -> bool:
        """Run — or share — a credential restoration for a logical request
        that began at ``start_epoch``. Returns whether replaying is
        worthwhile.

        Concurrent 401s await the same task instead of failing fast. The wait
        is bounded: a hung restorer (or a restorer awaiting itself through a
        restore-enabled nested call) fails this request with its original 401
        after the timeout — and the hung task is then CANCELLED. The slot is
        released only when the task actually finishes (its done-callback), so
        a replacement restoration can never overlap one that is still
        unwinding; if the task ignores cancellation past a shared grace
        deadline (one extra timeout from the first cancel request), the slot
        is hard-abandoned so the client can still recover. Restorer
        exceptions count as "not restored" so the original 401 is preserved.
        """
        if self._restoration_epoch != start_epoch:
            # Another request completed a restoration after this one began —
            # its outcome applies to our credentials too.
            return self._last_restore_outcome

        if self._restoration_task is None:
            restorer = self._credential_restorer
            if restorer is None:
                return False

            async def _run() -> bool:
                try:
                    return bool(await restorer())
                except Exception:
                    logger.warning(
                        "Credential restorer raised; propagating the original 401",
                        exc_info=True,
                    )
                    return False

            task = asyncio.get_running_loop().create_task(_run())

            def _finish(done: asyncio.Task[bool]) -> None:
                # Identity guard: an abandoned restoration settling late must
                # not bump the epoch, clobber a newer restoration, or clear a
                # slot it no longer owns.
                if self._restoration_task is not done:
                    return
                # The slot is released only HERE, when the task has truly
                # finished — including finishing its cancellation unwind — so
                # a replacement restoration can never overlap a dying one.
                self._restoration_task = None
                if not done.cancelled():
                    # A task that completed normally produced a real outcome —
                    # even one that suppressed cancellation to finish its
                    # rotation atomically. Record it.
                    self._restoration_epoch += 1
                    self._last_restore_outcome = done.result()

            task.add_done_callback(_finish)
            self._restoration_task = task
            self._restoration_cancel_deadline = None

        awaited = self._restoration_task
        try:
            # shield: one waiter timing out must not cancel the restoration
            # for the other waiters.
            outcome = await asyncio.wait_for(
                asyncio.shield(awaited), timeout=self._credential_restore_timeout
            )
        except asyncio.TimeoutError:
            logger.warning("Credential restoration timed out; propagating the original 401")
            if self._restoration_task is awaited:
                now = asyncio.get_running_loop().time()
                if self._restoration_cancel_deadline is None:
                    # First strike: cancel, but KEEP the slot occupied — it is
                    # released by the done-callback when the task finishes
                    # unwinding. Clearing it here would let a new restoration
                    # start while this one is still running its cancellation
                    # path (with rotating refresh tokens, a double rotation).
                    # New 401s arriving during the unwind observe the
                    # cancellation and degrade to their original 401s. The
                    # deadline is SHARED: siblings timing out alongside this
                    # waiter see it already set and still in the future, so
                    # they cannot mistake a just-requested cancellation for an
                    # ignored one and abandon the slot early.
                    self._restoration_cancel_deadline = now + self._credential_restore_timeout
                    awaited.cancel()
                elif now >= self._restoration_cancel_deadline:
                    # A full extra timeout has passed since cancellation was
                    # requested and the task ignored it (broken by contract —
                    # it neither finished nor honored the cancel).
                    # Hard-abandon the slot so the client can still recover;
                    # overlap is now possible, but only against that broken
                    # restorer.
                    self._restoration_task = None
            outcome = False
        except asyncio.CancelledError:
            # The shared task was cancelled by another waiter's timeout —
            # "not restored" for us too. Anything else (our own caller being
            # cancelled) must keep propagating.
            if not awaited.cancelled():
                raise
            outcome = False

        return outcome

    async def _ensure_session(self) -> aiohttp.ClientSession:
        if self._session is None or self._session.closed:
            self._session = aiohttp.ClientSession(
                timeout=self._timeout,
                # Cookies are managed explicitly: _auth_headers attaches them,
                # _capture_cookies harvests rotations into the token slot.
                # aiohttp's default CookieJar would ADDITIONALLY store every
                # response's Set-Cookie and silently re-attach it to later
                # requests to the same host — defeating per-call cookie-override
                # isolation on shared server clients (one caller's rotated
                # cookie would ride another caller's next request).
                cookie_jar=aiohttp.DummyCookieJar(),
                headers={
                    "Content-Type": "application/json",
                    "Accept": "application/json",
                },
            )
        return self._session

    async def close(self) -> None:
        """Close the HTTP session."""
        if self._session and not self._session.closed:
            await self._session.close()
            self._session = None

    async def __aenter__(self) -> "LightconeHttp":
        await self._ensure_session()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        await self.close()

    async def raw_post(self, url: str, body: Any) -> Any:
        """POST an arbitrary JSON body without ApiResponse parsing."""
        session = await self._ensure_session()
        async with session.post(url, json=body) as response:
            if 200 <= response.status < 300:
                try:
                    return await response.json()
                except (
                    ValueError,
                    json.JSONDecodeError,
                    aiohttp.ContentTypeError,
                ) as error:
                    raise HttpError.request(
                        f"Failed to parse response: {error}"
                    ) from error

            body_text = await response.text()
            raise self._map_status_error(
                response.status, body_text or "", response.headers
            )

    async def get(
        self,
        path: str,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
        *,
        params: Optional[dict[str, str]] = None,
        allow_credential_restore: bool = True,
    ) -> Any:
        """Make a GET request with user auth cookie injection.

        Pass ``allow_credential_restore=False`` for SDK calls made from
        inside a credential restorer: a restore-enabled call there awaits its
        own restoration and only the restoration timeout rescues it.
        """
        return await self._request_with_retry(
            "GET",
            path,
            retry_policy=retry_policy,
            auth_mode=_AuthMode.COOKIE,
            params=params,
            allow_credential_restore=allow_credential_restore,
        )

    async def get_with_cookies(
        self,
        path: str,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
        *,
        cookie_header: str,
        params: Optional[dict[str, str]] = None,
    ) -> Any:
        """Make a GET request forwarding an explicit per-call raw ``Cookie`` header.

        The header (e.g. ``"privy-token=…; lightcone-token=…"``) is sent verbatim.
        Intended for server-side cookie forwarding (SSR / server functions) where
        the per-request browser cookies can't propagate to the SDK's process-wide
        cookie store. Bypasses both the stored ``auth_token`` and the response-side
        ``Set-Cookie`` capture so per-call overrides never mutate shared state.
        """
        return await self._request_with_retry(
            "GET",
            path,
            retry_policy=retry_policy,
            auth_mode=_AuthMode.COOKIE_OVERRIDE,
            cookie_header_override=cookie_header,
            params=params,
            # Forwarded per-user cookies are outside the global credential
            # machinery: the process-wide restorer can't mint a new cookie
            # for THIS user, and a replay would resend the same stale header.
            allow_credential_restore=False,
        )

    async def post(
        self,
        path: str,
        body: Any,
        retry_policy: RetryPolicy = RetryPolicy.NONE,
        *,
        allow_credential_restore: bool = True,
    ) -> Any:
        """Make a POST request with user auth cookie injection.

        ``allow_credential_restore=False`` disables the transport's 401
        restore-and-replay. For credential-management endpoints (login,
        logout): they are the machinery restoration would re-run, so
        replaying them after restoring credentials is at best a no-op and at
        worst re-consumes single-use state (the login nonce is consumed
        server-side before the signature is verified, so a replayed login
        deterministically fails).
        """
        return await self._request_with_retry(
            "POST",
            path,
            retry_policy=retry_policy,
            auth_mode=_AuthMode.COOKIE,
            allow_credential_restore=allow_credential_restore,
            json=body,
        )

    async def delete(
        self,
        path: str,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
    ) -> Any:
        """Make a DELETE request with user auth cookie injection."""
        return await self._request_with_retry(
            "DELETE",
            path,
            retry_policy=retry_policy,
            auth_mode=_AuthMode.COOKIE,
        )

    async def post_with_cookies(
        self,
        path: str,
        body: Any,
        retry_policy: RetryPolicy = RetryPolicy.NONE,
        *,
        cookie_header: str,
    ) -> Any:
        """Make a POST request forwarding an explicit per-call Cookie header."""
        return await self._request_with_retry(
            "POST",
            path,
            retry_policy=retry_policy,
            auth_mode=_AuthMode.COOKIE_OVERRIDE,
            cookie_header_override=cookie_header,
            allow_credential_restore=False,
            json=body,
        )

    async def delete_with_cookies(
        self,
        path: str,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
        *,
        cookie_header: str,
    ) -> Any:
        """Make a DELETE request forwarding an explicit per-call Cookie header."""
        return await self._request_with_retry(
            "DELETE",
            path,
            retry_policy=retry_policy,
            auth_mode=_AuthMode.COOKIE_OVERRIDE,
            cookie_header_override=cookie_header,
            allow_credential_restore=False,
        )

    async def _request_with_retry(
        self,
        method: str,
        path: str,
        *,
        retry_policy: RetryPolicy = RetryPolicy.IDEMPOTENT,
        auth_mode: _AuthMode,
        cookie_header_override: Optional[str] = None,
        params: Optional[dict[str, str]] = None,
        allow_credential_restore: bool = True,
        **kwargs: Any,
    ) -> Any:
        """Make an HTTP request with retry logic and ApiResponse unwrapping."""
        # ``NONE`` still means "no transport retries"; it runs through the
        # same loop with zero retry attempts so the credential-restore path
        # below covers every request. It ALSO means "never auto-replay":
        # non-idempotent mutations declare themselves via ``RetryPolicy.NONE``,
        # so a 401 still triggers restoration (healing the session for the
        # caller's next attempt) but the 401 propagates instead of replaying.
        replay_allowed = not retry_policy.is_none()
        config = retry_policy.resolve_config() or RetryConfig(
            max_retries=0, retryable_statuses=set()
        )

        # One request id and one body serialization per LOGICAL request:
        # transport retries and the auth replay resend the same id (tracing
        # correlation, and the hook for future server-side idempotency) and
        # the same bytes (a mutable body can't drift between attempts).
        request_id = str(uuid.uuid4())
        json_body = kwargs.pop("json", None)
        if json_body is not None:
            kwargs["data"] = json.dumps(json_body)

        start_epoch = self._restoration_epoch
        credentials_restored = False
        attempt = 0

        while True:
            try:
                return await self._send_and_parse(
                    method,
                    path,
                    auth_mode=auth_mode,
                    cookie_header_override=cookie_header_override,
                    params=params,
                    request_id=request_id,
                    **kwargs,
                )
            except _HttpStatusError as error:
                # On the first 401, give the host a chance to restore its
                # credentials (e.g. refresh an auth session) — at most once
                # per logical request, shared with any concurrent requests
                # (see _restore_credentials_shared), and only for requests to
                # the API origin whose endpoint allows it (login/logout opt
                # out). The replay itself additionally requires the request
                # to have declared itself retry-safe.
                if (
                    not credentials_restored
                    and allow_credential_restore
                    and error.status == 401
                    and self._is_api_origin(path)
                ):
                    credentials_restored = True
                    restored = await self._restore_credentials_shared(start_epoch)
                    if restored and replay_allowed:
                        logger.debug(
                            "Credentials restored; replaying request to %s",
                            self._resolve_url(path),
                        )
                        continue

                should_retry = error.status in config.retryable_statuses
                if should_retry and attempt < config.max_retries:
                    delay_ms = _retry_after_ms(error.headers)
                    delay = (
                        delay_ms / 1000.0
                        if delay_ms is not None
                        else delay_for_attempt(attempt, config)
                    )
                    logger.debug(
                        "Retrying request to %s (attempt %d/%d, delay %.1fs)",
                        self._resolve_url(path),
                        attempt + 1,
                        config.max_retries,
                        delay,
                    )
                    attempt += 1
                    await asyncio.sleep(delay)
                    continue
                self._raise_status_error(error)
            except ApiRejected:
                raise
            except HttpError as error:
                should_retry = False

                if error.kind == HttpErrorKind.SERVER_ERROR:
                    should_retry = (
                        error.status is not None
                        and error.status in config.retryable_statuses
                    )
                elif error.kind == HttpErrorKind.RATE_LIMITED:
                    should_retry = 429 in config.retryable_statuses
                elif error.kind == HttpErrorKind.TIMEOUT:
                    should_retry = True

                if should_retry and attempt < config.max_retries:
                    delay = delay_for_attempt(attempt, config)
                    logger.debug(
                        "Retrying request to %s (attempt %d/%d, delay %.1fs)",
                        self._resolve_url(path),
                        attempt + 1,
                        config.max_retries,
                        delay,
                    )
                    attempt += 1
                    await asyncio.sleep(delay)
                    continue
                raise
            except asyncio.TimeoutError:
                if attempt < config.max_retries:
                    delay = delay_for_attempt(attempt, config)
                    attempt += 1
                    await asyncio.sleep(delay)
                    continue
                raise HttpError.timeout() from None
            except aiohttp.ClientError as error:
                retryable = isinstance(
                    error, aiohttp.ClientConnectorError
                ) and not isinstance(error, aiohttp.ClientSSLError)
                if retryable and attempt < config.max_retries:
                    delay = delay_for_attempt(attempt, config)
                    attempt += 1
                    await asyncio.sleep(delay)
                    continue
                raise HttpError.request(str(error)) from error

    async def _send_and_parse(
        self,
        method: str,
        path: str,
        *,
        auth_mode: _AuthMode,
        cookie_header_override: Optional[str] = None,
        params: Optional[dict[str, str]] = None,
        request_id: str,
        **kwargs: Any,
    ) -> Any:
        payload = await self._send_request(
            method,
            path,
            auth_mode=auth_mode,
            cookie_header_override=cookie_header_override,
            params=params,
            request_id=request_id,
            **kwargs,
        )
        return self._parse_api_response(payload, request_id)

    @staticmethod
    def _parse_api_response(payload: Any, request_id: str) -> Any:
        """Unwrap an API response or raise ApiRejected with the request id."""
        if not isinstance(payload, dict) or payload.get("status") not in {
            "success",
            "error",
        }:
            return payload

        parsed = ApiResponse.from_dict(payload)
        if parsed.status == "success":
            return parsed.body

        details = parsed.details or ApiRejectedDetails(reason="Unknown API rejection")
        raise ApiRejected(details.with_request_id(request_id))

    async def _send_request(
        self,
        method: str,
        path: str,
        *,
        auth_mode: _AuthMode,
        cookie_header_override: Optional[str] = None,
        params: Optional[dict[str, str]] = None,
        request_id: str,
        **kwargs: Any,
    ) -> Any:
        """Send one attempt and return the raw decoded JSON payload.

        The request id and pre-serialized body come from the logical request
        (``_request_with_retry``) so every attempt sends identical bytes.
        """
        session = await self._ensure_session()
        headers = dict(kwargs.pop("headers", {}))
        headers["x-request-id"] = request_id
        # Cookie injection is origin-gated: session credentials only ride to
        # the configured API origin, never to an arbitrary absolute URL a
        # caller supplies.
        if self._is_api_origin(path):
            headers.update(self._auth_headers(auth_mode, cookie_header_override))

        async with session.request(
            method,
            self._resolve_url(path),
            headers=headers,
            params=params,
            # The API never legitimately redirects; following one would let a
            # redirect target observe the request (and, before this guard,
            # receive the forwarded cookie header) while origin checks still
            # saw the original URL. A 3xx therefore surfaces as an error.
            allow_redirects=False,
            **kwargs,
        ) as response:
            if 200 <= response.status < 300:
                # Per-call overrides must not mutate the shared cookie store —
                # response Set-Cookie headers from a forwarded auth_token would
                # otherwise leak into the SDK's process-wide token slot.
                if auth_mode is not _AuthMode.COOKIE_OVERRIDE:
                    self._capture_cookies(response.headers)
                try:
                    return await response.json()
                except (
                    ValueError,
                    json.JSONDecodeError,
                    aiohttp.ContentTypeError,
                ) as error:
                    raise HttpError.request(
                        f"Failed to parse response: {error}"
                    ) from error

            body_text = await response.text()
            raise _HttpStatusError(
                response.status, body_text or "", response.headers, request_id
            )

    def _resolve_url(self, path: str) -> str:
        if path.startswith("http://") or path.startswith("https://"):
            return path
        return f"{self._base_url}{path}"

    def _is_api_origin(self, url: str) -> bool:
        """True when ``url`` shares the configured API origin.

        Session credentials are only injected — and the credential restorer
        only consulted — for same-origin requests: a foreign URL that answers
        401 must be able to neither trigger a restoration nor receive the
        restored cookie on a replay. Unparseable URLs count as foreign.
        """
        from urllib.parse import urlparse

        request_url = urlparse(self._resolve_url(url))
        base_url = urlparse(self._base_url)
        if not request_url.scheme or not request_url.hostname:
            return False
        default_ports = {"http": 80, "https": 443}
        request_port = request_url.port or default_ports.get(request_url.scheme)
        base_port = base_url.port or default_ports.get(base_url.scheme)
        return (
            request_url.scheme == base_url.scheme
            and request_url.hostname == base_url.hostname
            and request_port == base_port
        )

    def _auth_headers(
        self,
        auth_mode: _AuthMode,
        cookie_header_override: Optional[str] = None,
    ) -> dict[str, str]:
        headers: dict[str, str] = {}
        if auth_mode == _AuthMode.COOKIE_OVERRIDE:
            if cookie_header_override:
                # Forward the supplied Cookie header verbatim (may carry
                # privy-token and/or lightcone-token).
                headers["Cookie"] = cookie_header_override
        elif auth_mode == _AuthMode.COOKIE and self._auth_token:
            headers["Cookie"] = f"lightcone-token={self._auth_token}"
        return headers

    def _capture_cookies(self, headers: aiohttp.typedefs.LooseHeaders) -> None:
        set_cookie_headers = []
        if hasattr(headers, "getall"):
            set_cookie_headers = list(headers.getall("set-cookie", []))
        for cookie_header in set_cookie_headers:
            if cookie_header.startswith("lightcone-token="):
                token = cookie_header.split("lightcone-token=", 1)[1].split(";", 1)[0]
                if token:
                    self._auth_token = token

    def _map_status_error(
        self,
        status: int,
        message: str,
        headers: Optional[aiohttp.typedefs.LooseHeaders] = None,
    ) -> HttpError:
        """Map HTTP status to HttpError."""
        if status == 401:
            return HttpError.unauthorized(message)
        if status == 404:
            return HttpError.not_found(message)
        if status == 429:
            return HttpError.rate_limited(
                message or "Rate limited",
                retry_after_ms=_retry_after_ms(headers),
            )
        if 400 <= status <= 499:
            return HttpError.bad_request(message)
        return HttpError.server_error(message, status)

    def _raise_status_error(self, error: _HttpStatusError) -> None:
        parsed = self._parse_rejection_body(error.body, error.request_id, error.status)
        if parsed is not None:
            raise parsed
        raise self._map_status_error(error.status, error.body, error.headers)

    @staticmethod
    def _parse_rejection_body(
        body: str, request_id: str, http_status: int
    ) -> ApiRejected | None:
        try:
            payload = json.loads(body)
        except (ValueError, json.JSONDecodeError):
            return None
        if not isinstance(payload, dict) or payload.get("status") != "error":
            return None
        parsed = ApiResponse.from_dict(payload)
        details = parsed.details or ApiRejectedDetails(reason="Unknown API rejection")
        return ApiRejected(
            details.with_request_id(request_id).with_http_status(http_status)
        )


def _retry_after_ms(headers: Optional[aiohttp.typedefs.LooseHeaders]) -> Optional[int]:
    if headers is None:
        return None

    def _header(name: str) -> Optional[str]:
        if hasattr(headers, "get"):
            value = headers.get(name)
            return str(value) if value is not None else None
        return None

    retry_after_ms = _header("retry-after-ms")
    if retry_after_ms:
        try:
            return int(retry_after_ms)
        except ValueError:
            return None

    retry_after = _header("retry-after")
    if retry_after:
        try:
            return int(float(retry_after) * 1000)
        except ValueError:
            return None

    return None


__all__ = ["LightconeHttp"]
