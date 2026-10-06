"""HTTP retry and structured error handling tests."""

import asyncio
from collections.abc import Awaitable, Callable
from types import SimpleNamespace

import pytest
from aiohttp import web

from lightcone_sdk.auth.client import Auth
from lightcone_sdk.client import LightconeClientBuilder
from lightcone_sdk.error import ApiRejected, HttpError, is_unauthorized
from lightcone_sdk.http.client import LightconeHttp
from lightcone_sdk.http.retry import RetryConfig, RetryPolicy


async def _start_server(
    handler: Callable[[web.Request], Awaitable[web.Response]],
) -> tuple[str, Callable[[], Awaitable[None]]]:
    app = web.Application()
    app.router.add_route("*", "/{tail:.*}", handler)
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "127.0.0.1", 0)
    await site.start()
    sockets = site._server.sockets  # type: ignore[union-attr]
    port = sockets[0].getsockname()[1]
    return f"http://127.0.0.1:{port}", runner.cleanup


async def _server(
    responses: list[tuple[int, str]],
    cookies_seen: list[str | None] | None = None,
    location: str | None = None,
) -> tuple[str, Callable[[], int], Callable[[], Awaitable[None]]]:
    attempts = 0
    queue = list(responses)

    async def handler(request: web.Request) -> web.Response:
        nonlocal attempts
        attempts += 1
        if cookies_seen is not None:
            cookies_seen.append(request.headers.get("Cookie"))
        status, body = (
            queue.pop(0)
            if queue
            else (
                500,
                '{"status":"error","error_details":{"reason":"unexpected extra request"}}',
            )
        )
        headers = {"Location": location} if location is not None else None
        return web.Response(
            status=status, text=body, content_type="application/json", headers=headers
        )

    base_url, cleanup = await _start_server(handler)

    return (
        base_url,
        lambda: attempts,
        cleanup,
    )


def _fast_retry(statuses: set[int]) -> RetryPolicy:
    return RetryPolicy.custom(
        RetryConfig(
            max_retries=1,
            initial_delay_ms=0,
            max_delay_ms=0,
            backoff_factor=1,
            jitter=False,
            retryable_statuses=statuses,
        )
    )


@pytest.mark.asyncio
async def test_http_200_error_envelope_is_rejected() -> None:
    base_url, _, cleanup = await _server(
        [
            (
                200,
                '{"status":"error","error_details":{"reason":"invalid exact ratio","rejection_code":"PRICE_NOT_EXACTLY_REPRESENTABLE"}}',
            )
        ]
    )
    client = LightconeHttp(base_url)
    try:
        with pytest.raises(ApiRejected) as raised:
            await client.get("/test", RetryPolicy.NONE)
        assert raised.value.details.rejection_code is not None
        assert (
            raised.value.details.rejection_code.wire_name()
            == "PRICE_NOT_EXACTLY_REPRESENTABLE"
        )
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_structured_500_returns_api_rejected_details() -> None:
    base_url, _, cleanup = await _server(
        [
            (
                500,
                '{"status":"error","error_details":{"reason":"engine failed","error_code":"ENGINE","error_log_id":"LCERR_500"}}',
            )
        ]
    )
    client = LightconeHttp(base_url)

    try:
        with pytest.raises(ApiRejected) as raised:
            await client.get("/test", RetryPolicy.IDEMPOTENT)
        assert raised.value.details.reason == "engine failed"
        assert raised.value.details.error_code == "ENGINE"
        assert raised.value.details.error_log_id == "LCERR_500"
        assert raised.value.details.request_id is not None
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_custom_retry_policy_retries_raw_409_status() -> None:
    base_url, attempts, cleanup = await _server(
        [
            (
                409,
                '{"status":"error","error_details":{"reason":"nonce mismatch","error_code":"NONCE_MISMATCH"}}',
            ),
            (200, '{"status":"success","body":{"ok":true}}'),
        ]
    )
    client = LightconeHttp(base_url)

    try:
        body = await client.get("/retry", _fast_retry({409}))
        assert body == {"ok": True}
        assert attempts() == 2
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_delete_helpers_default_to_idempotent_retry() -> None:
    retry_error = '{"status":"error","error_details":{"reason":"retry"}}'
    success = '{"status":"success","body":{"ok":true}}'
    base_url, attempts, cleanup = await _server(
        [(503, retry_error), (200, success), (503, retry_error), (200, success)]
    )
    client = LightconeHttp(base_url)

    try:
        assert await client.delete("/favorite") == {"ok": True}
        assert await client.delete_with_cookies(
            "/favorite", cookie_header="lightcone-token=test"
        ) == {"ok": True}
        assert attempts() == 4
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_custom_retry_policy_does_not_retry_429_when_excluded() -> None:
    base_url, attempts, cleanup = await _server(
        [
            (
                429,
                '{"status":"error","error_details":{"reason":"rate limited","error_code":"RATE_LIMITED","error_log_id":"LCERR_429"}}',
            ),
            (200, '{"status":"success","body":{"ok":true}}'),
        ]
    )
    client = LightconeHttp(base_url)

    try:
        with pytest.raises(ApiRejected) as raised:
            await client.get("/retry", _fast_retry({503}))
        assert raised.value.details.reason == "rate limited"
        assert raised.value.details.error_code == "RATE_LIMITED"
        assert raised.value.details.error_log_id == "LCERR_429"
        assert attempts() == 1
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_retry_exhaustion_preserves_structured_503_details() -> None:
    base_url, attempts, cleanup = await _server(
        [
            (
                503,
                '{"status":"error","error_details":{"reason":"temporarily unavailable","error_code":"UNAVAILABLE","error_log_id":"LCERR_503A"}}',
            ),
            (
                503,
                '{"status":"error","error_details":{"reason":"still unavailable","error_code":"UNAVAILABLE","error_log_id":"LCERR_503B"}}',
            ),
        ]
    )
    client = LightconeHttp(base_url)

    try:
        with pytest.raises(ApiRejected) as raised:
            await client.get("/retry", _fast_retry({503}))
        assert raised.value.details.reason == "still unavailable"
        assert raised.value.details.error_log_id == "LCERR_503B"
        assert attempts() == 2
    finally:
        await client.close()
        await cleanup()


# ── Credential restorer (401 → restore → replay) ─────────────────────────────


class _StubRestorer:
    def __init__(self, restored: bool):
        self._restored = restored
        self.calls = 0

    async def __call__(self) -> bool:
        self.calls += 1
        return self._restored


@pytest.mark.asyncio
async def test_unauthorized_with_restorer_replays_once_after_restore() -> None:
    base_url, attempts, cleanup = await _server(
        [
            (401, "Unauthorized"),
            (200, '{"status":"success","body":{"ok":true}}'),
        ]
    )
    client = LightconeHttp(base_url)
    restorer = _StubRestorer(restored=True)
    client.set_credential_restorer(restorer)

    try:
        body = await client.get("/me", RetryPolicy.IDEMPOTENT)
        assert body == {"ok": True}
        assert attempts() == 2
        assert restorer.calls == 1
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_no_retry_posts_restore_but_never_replay() -> None:
    # RetryPolicy.NONE declares the request non-idempotent (orders etc.):
    # a 401 still triggers restoration — healing the session for the caller's
    # next attempt — but the request itself is NEVER auto-replayed; the
    # original 401 propagates.
    base_url, attempts, cleanup = await _server([(401, "Unauthorized")])
    client = LightconeHttp(base_url)
    restorer = _StubRestorer(restored=True)
    client.set_credential_restorer(restorer)

    try:
        with pytest.raises(HttpError) as raised:
            await client.post("/order", {"side": "buy"}, RetryPolicy.NONE)
        assert is_unauthorized(raised.value)
        assert attempts() == 1
        assert restorer.calls == 1
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_unauthorized_without_restorer_propagates_unchanged() -> None:
    base_url, attempts, cleanup = await _server([(401, "Unauthorized")])
    client = LightconeHttp(base_url)

    try:
        with pytest.raises(HttpError) as raised:
            await client.get("/me", RetryPolicy.IDEMPOTENT)
        assert is_unauthorized(raised.value)
        assert attempts() == 1
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_unauthorized_with_failed_restore_propagates_without_replay() -> None:
    base_url, attempts, cleanup = await _server([(401, "Unauthorized")])
    client = LightconeHttp(base_url)
    restorer = _StubRestorer(restored=False)
    client.set_credential_restorer(restorer)

    try:
        with pytest.raises(HttpError) as raised:
            await client.get("/me", RetryPolicy.IDEMPOTENT)
        assert is_unauthorized(raised.value)
        assert attempts() == 1
        assert restorer.calls == 1
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_restorer_consulted_at_most_once_per_request() -> None:
    # Restore "succeeds" but the replay still 401s (e.g. the restored session
    # is rejected too) — the second 401 must propagate rather than loop
    # through the restorer again.
    base_url, attempts, cleanup = await _server(
        [
            (401, "Unauthorized"),
            (401, "Unauthorized"),
        ]
    )
    client = LightconeHttp(base_url)
    restorer = _StubRestorer(restored=True)
    client.set_credential_restorer(restorer)

    try:
        with pytest.raises(HttpError) as raised:
            await client.get("/me", RetryPolicy.IDEMPOTENT)
        assert is_unauthorized(raised.value)
        assert attempts() == 2
        assert restorer.calls == 1
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_enveloped_401_reports_unauthorized_via_http_status() -> None:
    base_url, _attempts, cleanup = await _server(
        [
            (
                401,
                '{"status":"error","error_details":{"reason":"session expired","error_code":"SESSION_EXPIRED"}}',
            ),
        ]
    )
    client = LightconeHttp(base_url)

    try:
        with pytest.raises(ApiRejected) as raised:
            await client.get("/me", RetryPolicy.IDEMPOTENT)
        assert raised.value.details.http_status == 401
        assert raised.value.details.reason == "session expired"
        assert is_unauthorized(raised.value)
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_foreign_origin_401_gets_no_cookie_and_no_restorer() -> None:
    # The client's API origin is server A; the request goes to server B.
    # B answering 401 must neither receive the session cookie nor trigger a
    # credential restoration (a foreign endpoint could otherwise phish the
    # restored cookie via the replay).
    api_url, _api_attempts, api_cleanup = await _server([])
    foreign_cookies: list[str | None] = []
    foreign_url, foreign_attempts, foreign_cleanup = await _server(
        [(401, "Unauthorized")], cookies_seen=foreign_cookies
    )
    client = LightconeHttp(api_url)
    client.set_auth_token("secret-token")
    restorer = _StubRestorer(restored=True)
    client.set_credential_restorer(restorer)

    try:
        with pytest.raises(HttpError) as raised:
            await client.get(f"{foreign_url}/me", RetryPolicy.IDEMPOTENT)
        assert is_unauthorized(raised.value)
        assert foreign_attempts() == 1
        assert restorer.calls == 0
        assert foreign_cookies[0] is None
    finally:
        await client.close()
        await api_cleanup()
        await foreign_cleanup()


@pytest.mark.asyncio
async def test_same_origin_request_still_sends_cookie() -> None:
    # Companion to the foreign-origin test: the gate must not over-block.
    cookies: list[str | None] = []
    base_url, attempts, cleanup = await _server(
        [(200, '{"status":"success","body":{"ok":true}}')], cookies_seen=cookies
    )
    client = LightconeHttp(base_url)
    client.set_auth_token("secret-token")

    try:
        body = await client.get("/me", RetryPolicy.IDEMPOTENT)
        assert body == {"ok": True}
        assert attempts() == 1
        assert cookies[0] == "lightcone-token=secret-token"
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_no_restore_post_skips_restorer_on_401() -> None:
    # Credential-management endpoints (login/logout) opt out of the
    # restore-and-replay: restoring credentials in order to log in is
    # circular, and a replayed login would re-consume single-use state.
    base_url, attempts, cleanup = await _server([(401, "Unauthorized")])
    client = LightconeHttp(base_url)
    restorer = _StubRestorer(restored=True)
    client.set_credential_restorer(restorer)

    try:
        with pytest.raises(HttpError) as raised:
            await client.post(
                "/api/auth/login_or_register_with_message",
                {"message": "m"},
                RetryPolicy.NONE,
                allow_credential_restore=False,
            )
        assert is_unauthorized(raised.value)
        assert attempts() == 1
        assert restorer.calls == 0
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_reentrant_restorer_terminates_and_runs_once() -> None:
    # A restorer that re-logins through the SDK: its nested request also
    # 401s. The client-wide single-flight flag must stop that nested 401
    # from starting a second restoration.
    base_url, attempts, cleanup = await _server(
        [(401, "Unauthorized"), (401, "Unauthorized")]
    )
    client = LightconeHttp(base_url)
    calls = 0

    async def reentrant_restorer() -> bool:
        nonlocal calls
        calls += 1
        try:
            # Restorer-internal SDK calls must use the no-restore variant: a
            # restore-enabled call here would await its own restoration and
            # only the restoration timeout would rescue it.
            await client.get(
                "/nested", RetryPolicy.NONE, allow_credential_restore=False
            )
        except Exception:
            pass
        return False

    client.set_credential_restorer(reentrant_restorer)

    try:
        with pytest.raises(HttpError) as raised:
            await client.get("/me", RetryPolicy.IDEMPOTENT)
        assert is_unauthorized(raised.value)
        # Outer request + the restorer's nested request; no replay and no
        # second restoration from the nested 401.
        assert attempts() == 2
        assert calls == 1
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_concurrent_401s_share_one_restoration() -> None:
    # Two requests hit expiry together: both must recover, sharing a single
    # restorer run — the second awaits the in-flight restoration instead of
    # failing fast.
    base_url, attempts, cleanup = await _server(
        [
            (401, "Unauthorized"),
            (401, "Unauthorized"),
            (200, '{"status":"success","body":{"ok":true}}'),
            (200, '{"status":"success","body":{"ok":true}}'),
        ]
    )
    client = LightconeHttp(base_url)
    calls = 0

    async def slow_restorer() -> bool:
        nonlocal calls
        calls += 1
        await asyncio.sleep(0.1)
        return True

    client.set_credential_restorer(slow_restorer)

    try:
        first, second = await asyncio.gather(
            client.get("/a", RetryPolicy.IDEMPOTENT),
            client.get("/b", RetryPolicy.IDEMPOTENT),
        )
        assert first == {"ok": True}
        assert second == {"ok": True}
        assert calls == 1
        assert attempts() == 4
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_hung_restorer_times_out_and_client_recovers() -> None:
    base_url, attempts, cleanup = await _server(
        [
            (401, "Unauthorized"),
            (401, "Unauthorized"),
            (200, '{"status":"success","body":{"ok":true}}'),
        ]
    )
    client = LightconeHttp(base_url)
    client._credential_restore_timeout = 0.2
    hung_calls = 0

    async def hung_restorer() -> bool:
        nonlocal hung_calls
        hung_calls += 1
        await asyncio.Event().wait()
        return True

    client.set_credential_restorer(hung_restorer)

    try:
        started = asyncio.get_running_loop().time()
        with pytest.raises(HttpError) as raised:
            await client.get("/me", RetryPolicy.IDEMPOTENT)
        assert is_unauthorized(raised.value)
        assert asyncio.get_running_loop().time() - started >= 0.15
        assert hung_calls == 1

        # The client is not stuck "restoring": a replacement restorer works.
        stub = _StubRestorer(restored=True)
        client.set_credential_restorer(stub)
        body = await client.get("/again", RetryPolicy.IDEMPOTENT)
        assert body == {"ok": True}
        assert stub.calls == 1
        assert attempts() == 3
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_restorer_exception_preserves_original_401() -> None:
    base_url, attempts, cleanup = await _server([(401, "Unauthorized")])
    client = LightconeHttp(base_url)
    calls = 0

    async def exploding_restorer() -> bool:
        nonlocal calls
        calls += 1
        raise RuntimeError("restorer exploded")

    client.set_credential_restorer(exploding_restorer)

    try:
        with pytest.raises(HttpError) as raised:
            await client.get("/me", RetryPolicy.IDEMPOTENT)
        # The auth error, not the restorer's failure, reaches the caller.
        assert is_unauthorized(raised.value)
        assert attempts() == 1
        assert calls == 1
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_api_redirects_are_not_followed() -> None:
    foreign_url, foreign_attempts, foreign_cleanup = await _server([])
    base_url, attempts, cleanup = await _server(
        [(302, "")], location=f"{foreign_url}/steal"
    )
    client = LightconeHttp(base_url)
    restorer = _StubRestorer(restored=True)
    client.set_credential_restorer(restorer)

    try:
        with pytest.raises(HttpError) as raised:
            await client.get("/me", RetryPolicy.NONE)
        assert not is_unauthorized(raised.value)
        assert attempts() == 1
        assert foreign_attempts() == 0
        assert restorer.calls == 0
    finally:
        await client.close()
        await cleanup()
        await foreign_cleanup()


@pytest.mark.asyncio
async def test_cookie_override_401_does_not_consult_restorer() -> None:
    base_url, attempts, cleanup = await _server([(401, "Unauthorized")])
    client = LightconeHttp(base_url)
    restorer = _StubRestorer(restored=True)
    client.set_credential_restorer(restorer)

    try:
        with pytest.raises(HttpError) as raised:
            await client.get_with_cookies(
                "/ssr", RetryPolicy.IDEMPOTENT, cookie_header="privy-token=stale"
            )
        assert is_unauthorized(raised.value)
        assert attempts() == 1
        assert restorer.calls == 0
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_ambient_cookie_jar_is_disabled() -> None:
    """A Set-Cookie on one response must not ride later requests.

    aiohttp's default CookieJar stores every Set-Cookie and re-attaches it to
    subsequent same-host requests, which would defeat cookie-override
    isolation at the wire level (the SDK-slot assertions elsewhere would still
    pass). The session uses DummyCookieJar so only _auth_headers decides what
    goes out.
    """
    cookies_seen: list[str | None] = []

    async def handler(request: web.Request) -> web.Response:
        cookies_seen.append(request.headers.get("Cookie"))
        response = web.Response(
            status=200,
            text='{"status":"success","body":{"ok":true}}',
            content_type="application/json",
        )
        if request.path == "/override":
            response.headers["Set-Cookie"] = "lightcone-token=other-users-rotated"
        return response

    app = web.Application()
    app.router.add_route("*", "/{tail:.*}", handler)
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "127.0.0.1", 0)
    await site.start()
    sockets = site._server.sockets  # type: ignore[union-attr]
    port = sockets[0].getsockname()[1]
    # "localhost", not 127.0.0.1: aiohttp's default jar refuses cookies from
    # bare-IP hosts, which would make this test pass even without the fix.
    client = LightconeHttp(f"http://localhost:{port}")

    try:
        await client.get_with_cookies(
            "/override",
            RetryPolicy.IDEMPOTENT,
            cookie_header="lightcone-token=forwarded",
        )
        await client.get("/plain", RetryPolicy.IDEMPOTENT)
        assert cookies_seen == ["lightcone-token=forwarded", None]
        assert not client.has_auth_token()
    finally:
        await client.close()
        await runner.cleanup()


@pytest.mark.asyncio
async def test_timed_out_restoration_is_cancelled_for_real() -> None:
    base_url, attempts, cleanup = await _server(
        [
            (401, "Unauthorized"),
            (401, "Unauthorized"),
        ]
    )
    client = LightconeHttp(base_url)
    client._credential_restore_timeout = 0.2
    observed = {"calls": 0, "cancelled": False}

    async def hung_restorer() -> bool:
        observed["calls"] += 1
        try:
            await asyncio.Event().wait()
        except asyncio.CancelledError:
            observed["cancelled"] = True
            raise
        return True

    client.set_credential_restorer(hung_restorer)

    try:
        # Stagger the joiner so the leader's timeout fires while the joiner is
        # still waiting: the leader CANCELS the shared restoration and the
        # joiner degrades to its original 401 instead of propagating the
        # cancellation to its caller.
        leader = asyncio.create_task(client.get("/a", RetryPolicy.IDEMPOTENT))
        await asyncio.sleep(0.1)
        joiner = asyncio.create_task(client.get("/b", RetryPolicy.IDEMPOTENT))
        results = await asyncio.gather(leader, joiner, return_exceptions=True)

        assert all(isinstance(result, HttpError) for result in results)
        assert all(is_unauthorized(result) for result in results)
        assert attempts() == 2
        assert observed["calls"] == 1
        assert observed["cancelled"] is True
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_cancel_suppressing_restorer_never_overlaps() -> None:
    """The slot is held until the cancelled task actually finishes.

    A restorer that suppresses cancellation to finish its work atomically
    completes normally after the leader's timeout. A request arriving during
    that window must NOT start a second restorer (no overlapping rotations);
    it joins the finishing task and can act on its real outcome.
    """
    base_url, attempts, cleanup = await _server(
        [
            (401, "Unauthorized"),
            (401, "Unauthorized"),
            (200, '{"status":"success","body":{"ok":true}}'),
        ]
    )
    client = LightconeHttp(base_url)
    client._credential_restore_timeout = 0.2
    calls = 0

    async def suppressing_restorer() -> bool:
        nonlocal calls
        calls += 1
        try:
            await asyncio.Event().wait()
        except asyncio.CancelledError:
            # Finish "atomically" instead of unwinding immediately.
            await asyncio.sleep(0.05)
            return True
        return True

    client.set_credential_restorer(suppressing_restorer)

    try:
        leader = asyncio.create_task(client.get("/a", RetryPolicy.IDEMPOTENT))
        # Join while the leader is still waiting; stay joined through the
        # leader's timeout and the suppression window.
        await asyncio.sleep(0.1)
        joiner = asyncio.create_task(client.get("/b", RetryPolicy.IDEMPOTENT))
        results = await asyncio.gather(leader, joiner, return_exceptions=True)

        # Leader timed out (401); the joiner saw the suppressed task complete
        # with True and replayed successfully. Exactly one restorer ran.
        assert calls == 1
        leader_result, joiner_result = results
        assert isinstance(leader_result, HttpError)
        assert is_unauthorized(leader_result)
        assert joiner_result == {"ok": True}
        assert attempts() == 3
    finally:
        await client.close()
        await cleanup()


# ── Auth logout error propagation ─────────────────────────────────────────────


@pytest.mark.asyncio
async def test_logout_failure_propagates_after_clearing_local_state() -> None:
    """The app's logout teardown gate reads this exception to decide whether
    the WebSocket may reconnect — a swallowed failure would let it restart
    with a still-valid server-side cookie."""
    base_url, attempts, cleanup = await _server(
        [(500, '{"status":"error","error_details":{"reason":"session store down"}}')]
    )
    http = LightconeHttp(base_url)
    http.set_auth_token("live-cookie")
    auth = Auth(SimpleNamespace(_http=http))

    try:
        with pytest.raises(ApiRejected):
            await auth.logout()
        # Local state was still cleared before the re-raise.
        assert not http.has_auth_token()
        assert auth.credentials() is None
        assert attempts() == 1
    finally:
        await http.close()
        await cleanup()


@pytest.mark.asyncio
async def test_logout_401_counts_as_success() -> None:
    base_url, _, cleanup = await _server([(401, "Unauthorized")])
    http = LightconeHttp(base_url)
    http.set_auth_token("stale-cookie")
    auth = Auth(SimpleNamespace(_http=http))

    try:
        await auth.logout()
        assert not http.has_auth_token()
    finally:
        await http.close()
        await cleanup()


@pytest.mark.asyncio
async def test_simultaneous_timeouts_share_one_cancellation_grace() -> None:
    """Reviewer repro for the per-waiter "second strike" bug.

    Two waiters that time out together must not abandon the just-cancelled
    slot: the second waiter's timeout is not evidence the task IGNORED
    cancellation — the first waiter only just requested it. Abandoning early
    let a third request start an overlapping restoration (calls == 2).
    """
    base_url, attempts, cleanup = await _server(
        [
            (401, "Unauthorized"),
            (401, "Unauthorized"),
            (401, "Unauthorized"),
            (200, '{"status":"success","body":{"ok":true}}'),
        ]
    )
    client = LightconeHttp(base_url)
    client._credential_restore_timeout = 0.2
    calls = 0

    async def suppressing_restorer() -> bool:
        nonlocal calls
        calls += 1
        try:
            await asyncio.Event().wait()
        except asyncio.CancelledError:
            # Finish "atomically": the unwind takes a while, then succeeds.
            await asyncio.sleep(0.1)
            return True
        return True

    client.set_credential_restorer(suppressing_restorer)

    try:
        waiter_a = asyncio.create_task(client.get("/a", RetryPolicy.IDEMPOTENT))
        waiter_b = asyncio.create_task(client.get("/b", RetryPolicy.IDEMPOTENT))
        # Both time out together at ~0.2: the first cancels and starts the
        # shared grace deadline; the second must leave the slot alone.
        await asyncio.sleep(0.25)
        # Mid-unwind: must join the dying task, not start restorer #2.
        joiner = asyncio.create_task(client.get("/c", RetryPolicy.IDEMPOTENT))
        results = await asyncio.gather(
            waiter_a, waiter_b, joiner, return_exceptions=True
        )

        assert calls == 1
        for result in results[:2]:
            assert isinstance(result, HttpError)
            assert is_unauthorized(result)
        # The suppressed task completed True; the joiner acted on it.
        assert results[2] == {"ok": True}
        assert attempts() == 4
    finally:
        await client.close()
        await cleanup()


async def _header_capture_server() -> (
    tuple[str, list[str | None], Callable[[], Awaitable[None]]]
):
    seen: list[str | None] = []

    async def handler(request: web.Request) -> web.Response:
        seen.append(request.headers.get("x-lightcone-api-key"))
        return web.Response(
            status=200,
            text='{"status":"success","body":{"ok":true}}',
            content_type="application/json",
        )

    base_url, cleanup = await _start_server(handler)
    return base_url, seen, cleanup


@pytest.mark.asyncio
async def test_api_key_rides_only_to_the_api_origin() -> None:
    base_url, seen, cleanup = await _header_capture_server()
    try:
        http = LightconeHttp(base_url, api_key="lc_local_key")
        assert http.has_api_key
        for route in ("submit", "cancel", "cancel-all"):
            await http.post(f"{base_url}/api/orders/{route}", {}, RetryPolicy.NONE)
        foreign = LightconeHttp("http://127.0.0.1:1", api_key="lc_local_key")
        await foreign.post(f"{base_url}/api/orders/submit", {}, RetryPolicy.NONE)
        await http.get(f"{base_url}/api/markets", RetryPolicy.NONE)
        await http.get(f"{base_url}/api/orders/submit", RetryPolicy.NONE)
        await http.post(f"{base_url}/api/auth/login", {}, RetryPolicy.NONE)
        assert seen == ["lc_local_key"] * 3 + [None] * 4
        await http.close()
        await foreign.close()
    finally:
        await cleanup()


@pytest.mark.asyncio
async def test_blank_api_key_sends_no_header() -> None:
    base_url, seen, cleanup = await _header_capture_server()
    try:
        http = LightconeHttp(base_url, api_key="   ")
        assert not http.has_api_key
        await http.get(f"{base_url}/api/markets", RetryPolicy.NONE)
        assert seen == [None]
        await http.close()
    finally:
        await cleanup()


def test_builder_builds_without_an_api_key() -> None:
    client = LightconeClientBuilder().base_url("http://127.0.0.1:1").build()
    assert not client._http.has_api_key
    assert not client.has_api_key


def test_builder_passes_the_api_key_to_the_http_client() -> None:
    client = (
        LightconeClientBuilder()
        .base_url("http://127.0.0.1:1")
        .api_key("lc_local_key")
        .build()
    )
    assert client._http.has_api_key
    assert client.has_api_key


def test_api_key_rejects_non_loopback_cleartext_origin() -> None:
    with pytest.raises(ValueError, match="HTTPS or a loopback HTTP origin"):
        LightconeHttp("http://api.example.com", api_key="test-key")
    assert LightconeHttp("http://127.0.0.1:3001", api_key="test-key").has_api_key
    assert LightconeHttp("https://api.example.com", api_key="test-key").has_api_key


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "header,delay",
    [
        ("60", 60000),
        ("0.0001", 1),
        (None, None),
        ("-1", None),
        ("NaN", None),
        ("5junk", None),
    ],
)
async def test_privy_unavailable_returns_first_rejection_and_retains_credentials(
    header, delay
):
    attempts = 0

    async def handler(request):
        nonlocal attempts
        attempts += 1
        return web.json_response(
            {
                "status": "error",
                "error_details": {
                    "reason": "Unavailable",
                    "error_code": "PRIVY_VERIFICATION_UNAVAILABLE",
                },
            },
            status=503,
            headers={"Retry-After": header} if header is not None else {},
        )

    base, cleanup = await _start_server(handler)
    http = LightconeHttp(base)
    auth = Auth(SimpleNamespace(_http=http))
    cached = object()
    auth._credentials = cached
    try:
        with pytest.raises(ApiRejected) as raised:
            await asyncio.wait_for(auth.check_session(), timeout=2)
        assert raised.value.details.is_privy_verification_unavailable()
        assert raised.value.details.retry_after_ms == delay
        assert auth._credentials is cached
        assert attempts == 1
        from lightcone_sdk.auth import LinkedIdentitySelector, RegisterPrivyRequest

        with pytest.raises(ApiRejected):
            await asyncio.wait_for(
                auth.register_privy(
                    RegisterPrivyRequest(
                        LinkedIdentitySelector(
                            type="email", email="fixture@example.test"
                        )
                    )
                ),
                timeout=2,
            )
        assert auth._credentials is cached
        assert attempts == 2
    finally:
        await http.close()
        await cleanup()


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "policy", [RetryPolicy.IDEMPOTENT, _fast_retry({503}), RetryPolicy.NONE]
)
@pytest.mark.parametrize("method", ["GET", "POST"])
async def test_privy_policy_matrix_has_one_send_and_no_restoration(policy, method):
    async def handler(_request):
        nonlocal attempts
        attempts += 1
        return web.json_response(
            {
                "status": "error",
                "error_details": {
                    "reason": "Unavailable",
                    "error_code": "PRIVY_VERIFICATION_UNAVAILABLE",
                },
            },
            status=503,
            headers={"Retry-After": "60"},
        )

    attempts = 0
    restores = 0

    async def restore():
        nonlocal restores
        restores += 1
        return True

    base, cleanup = await _start_server(handler)
    http = LightconeHttp(base)
    http.set_credential_restorer(restore)
    try:
        with pytest.raises(ApiRejected) as raised:
            call = (
                http.get("/test", policy)
                if method == "GET"
                else http.post("/test", {"signature": "original"}, policy)
            )
            await asyncio.wait_for(call, timeout=2)
        assert raised.value.details.is_privy_verification_unavailable()
        assert raised.value.details.retry_after_ms == 60000
        assert attempts == 1
        assert restores == 0
    finally:
        await http.close()
        await cleanup()


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "details",
    [123, {"reason": "Unavailable", "error_code": "OTHER", "rejection_code": 123}],
)
async def test_malformed_unrelated_503_keeps_generic_retry(details):
    import json

    base, attempts, cleanup = await _server(
        [
            (503, json.dumps({"status": "error", "error_details": details})),
            (200, '{"status":"success","body":{"ok":true}}'),
        ]
    )
    http = LightconeHttp(base)
    try:
        assert await http.get("/test", _fast_retry({503})) == {"ok": True}
        assert attempts() == 2
    finally:
        await http.close()
        await cleanup()


@pytest.mark.parametrize(
    "header,expected",
    [
        ("Wed, 21 Oct 2015 07:28:05 GMT", 5000),
        ("Wed, 21 Oct 2015 07:27:00 GMT", 0),
        ("Wed, invalid", None),
        ("Wednesday, 21-Oct-15 07:28:05 GMT", 5000),
        ("Wed Oct 21 07:28:05 2015", 5000),
        ("Thu Oct 01 07:28:05 2015", 0),
        ("Thu Oct  1 07:28:05 2015", 0),
        ("Wed, 31 Feb 2027 07:28:05 GMT", None),
        ("Oct 21 2099", None),
        ("Wed 1", None),
        ("Wed, 21 Oct 2015 07:28:05 +0000", None),
        ("Thu, 21 Oct 2015 07:28:05 GMT", None),
    ],
)
def test_http_date_retry_guidance_uses_response_time(header, expected):
    from lightcone_sdk.http.client import _retry_after_ms

    assert _retry_after_ms({"retry-after": header}, now_ms=1445412480000) == expected


@pytest.mark.asyncio
@pytest.mark.parametrize("operation", ["submit", "cancel", "cancel_all"])
async def test_public_order_operations_surface_one_privy_rejection(operation):
    from lightcone_sdk.domain.order import CancelAllBody, CancelBody
    from lightcone_sdk.shared.types import SubmitOrderRequest

    rules = {
        "orderbook_id": "11111111111111111111111111111111",
        "base_decimals": 8,
        "quote_decimals": 6,
        "price_decimals": 4,
        "trading_rules": {
            "base_size_decimals": 5,
            "max_price_decimals": 1,
            "max_price_significant_figures": 5,
            "integer_prices_always_allowed": True,
            "price_quantum": "0.1000",
            "price_quantum_raw": "1000",
            "base_size_quantum": "0.00001000",
            "base_size_quantum_raw": "1000",
        },
    }
    attempts = 0

    async def handler(request):
        nonlocal attempts
        if request.path.endswith("/decimals"):
            return web.json_response({"status": "success", "body": rules})
        attempts += 1
        return web.json_response(
            {
                "status": "error",
                "error_details": {
                    "reason": "Unavailable",
                    "error_code": "PRIVY_VERIFICATION_UNAVAILABLE",
                },
            },
            status=503,
            headers={"Retry-After": "60"},
        )

    base, cleanup = await _start_server(handler)
    client = LightconeClientBuilder().base_url(base).build()
    wallet = "11111111111111111111111111111111"
    try:
        with pytest.raises(ApiRejected) as raised:
            if operation == "submit":
                call = client.orders().submit(
                    SubmitOrderRequest(
                        maker=wallet,
                        nonce=0,
                        salt=0,
                        market_pubkey=wallet,
                        base_token=wallet,
                        quote_token=wallet,
                        side=0,
                        amount_in=15_185_088,
                        amount_out=123_456_000,
                        expiration=0,
                        signature="fixture",
                        orderbook_id=wallet,
                    )
                )
            elif operation == "cancel":
                call = client.orders().cancel(
                    CancelBody(order_hash="fixture", maker=wallet, signature="fixture")
                )
            else:
                call = client.orders().cancel_all(
                    CancelAllBody(
                        user_pubkey=wallet,
                        orderbook_id=wallet,
                        signature="fixture",
                        timestamp=0,
                        salt="fixture",
                    )
                )
            await asyncio.wait_for(call, timeout=2)
        assert raised.value.details.is_privy_verification_unavailable()
        assert raised.value.details.retry_after_ms == 60000
        assert attempts == 1
    finally:
        await client.close()
        await cleanup()


@pytest.mark.asyncio
async def test_http_date_is_anchored_before_delayed_body(monkeypatch):
    import aiohttp

    import lightcone_sdk.http.client as transport

    now = 1445412480.0
    monkeypatch.setattr(transport.time, "time", lambda: now)
    original_text = aiohttp.ClientResponse.text

    async def delayed_text(response, *args, **kwargs):
        nonlocal now
        now += 2
        return await original_text(response, *args, **kwargs)

    monkeypatch.setattr(aiohttp.ClientResponse, "text", delayed_text)

    async def handler(request):
        return web.json_response(
            {
                "status": "error",
                "error_details": {
                    "reason": "Unavailable",
                    "error_code": "PRIVY_VERIFICATION_UNAVAILABLE",
                },
            },
            status=503,
            headers={"Retry-After": "Wed, 21 Oct 2015 07:28:05 GMT"},
        )

    base, cleanup = await _start_server(handler)
    http = LightconeHttp(base)
    try:
        with pytest.raises(ApiRejected) as caught:
            await http.get("/test", RetryPolicy.NONE)
        assert caught.value.details.retry_after_ms == 5000
    finally:
        await http.close()
        await cleanup()


@pytest.mark.parametrize(
    "milliseconds,expected",
    [("1.1", 2), ("0", 0), ("bad", 5000), ("9007199254740992", 5000)],
)
def test_millisecond_guidance_precedence(milliseconds, expected):
    from lightcone_sdk.http.client import _retry_after_ms

    assert (
        _retry_after_ms({"retry-after-ms": milliseconds, "retry-after": "5"})
        == expected
    )


def test_obsolete_http_year_uses_receipt_year():
    from datetime import UTC, datetime

    from lightcone_sdk.http.client import _retry_after_ms

    now = datetime(2026, 1, 1, tzinfo=UTC).timestamp() * 1000
    expected = int(datetime(2070, 1, 1, tzinfo=UTC).timestamp() * 1000 - now)
    assert (
        _retry_after_ms({"retry-after": "Wednesday, 01-Jan-70 00:00:00 GMT"}, now)
        == expected
    )


@pytest.mark.asyncio
async def test_malformed_optional_fields_cannot_enable_privy_replay():
    base, attempts, cleanup = await _server(
        [
            (
                503,
                '{"status":"error","error_details":{"reason":"Unavailable","error_code":"PRIVY_VERIFICATION_UNAVAILABLE","rejection_code":123}}',
            )
        ]
    )
    http = LightconeHttp(base)
    try:
        with pytest.raises(ApiRejected) as caught:
            await http.get("/test", _fast_retry({503}))
        assert caught.value.details.is_privy_verification_unavailable()
        assert attempts() == 1
    finally:
        await http.close()
        await cleanup()
