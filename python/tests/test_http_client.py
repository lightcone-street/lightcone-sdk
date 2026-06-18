"""HTTP retry and structured error handling tests."""

from collections.abc import Awaitable, Callable

import pytest
from aiohttp import web

from lightcone_sdk.error import ApiRejected
from lightcone_sdk.http.client import LightconeHttp
from lightcone_sdk.http.retry import RetryConfig, RetryPolicy


async def _server(
    responses: list[tuple[int, str]],
) -> tuple[str, Callable[[], int], Callable[[], Awaitable[None]]]:
    attempts = 0
    queue = list(responses)

    async def handler(_request: web.Request) -> web.Response:
        nonlocal attempts
        attempts += 1
        status, body = (
            queue.pop(0)
            if queue
            else (
                500,
                '{"status":"error","error_details":{"reason":"unexpected extra request"}}',
            )
        )
        return web.Response(status=status, text=body, content_type="application/json")

    app = web.Application()
    app.router.add_route("*", "/{tail:.*}", handler)
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "127.0.0.1", 0)
    await site.start()
    sockets = site._server.sockets  # type: ignore[union-attr]
    port = sockets[0].getsockname()[1]

    return (
        f"http://127.0.0.1:{port}",
        lambda: attempts,
        runner.cleanup,
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
