"""Native client auth frames preserve the WebSocket transport contract."""

import asyncio
from types import SimpleNamespace

import pytest
from aiohttp import WSMsgType, web

from lightcone_sdk.auth.client import Auth
from lightcone_sdk.error import ApiRejected
from lightcone_sdk.http.client import LightconeHttp
from lightcone_sdk.ws import MessageInType, WsConfig, WsEventType
from lightcone_sdk.ws.client import WsClient
from lightcone_sdk.ws.subscriptions import MarketParams


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "code,status",
    [
        ("TOKEN_REVOCATION_UNAVAILABLE", 503),
        ("TOKEN_REVOCATION_RECOVERY_UNAVAILABLE", 503),
        ("TOKEN_REVOCATION_FENCE_PENDING", 503),
        ("AMBIGUOUS_LIGHTCONE_TOKEN", 400),
    ],
)
async def test_logout_presents_token_and_preserves_revocation_error(code, status):
    cookies_seen = []

    async def handler(request):
        cookies_seen.append(request.headers.get("Cookie"))
        return web.json_response(
            {
                "status": "error",
                "error_details": {"reason": "incomplete", "error_code": code},
            },
            status=status,
        )

    app = web.Application()
    app.router.add_post("/api/auth/logout", handler)
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "127.0.0.1", 0)
    await site.start()
    port = site._server.sockets[0].getsockname()[1]
    http = LightconeHttp(f"http://127.0.0.1:{port}")
    http.set_auth_token("live-cookie")
    auth = Auth(SimpleNamespace(_http=http))
    try:
        with pytest.raises(ApiRejected) as raised:
            await auth.logout()
        assert raised.value.details.error_code == code
        assert cookies_seen == ["lightcone-token=live-cookie"]
        assert not http.has_auth_token()
    finally:
        await http.close()
        await runner.cleanup()


@pytest.mark.asyncio
async def test_anonymous_frames_keep_public_transport_and_replay_state():
    connections = 0
    subscriptions = []

    async def handler(request):
        nonlocal connections
        connections += 1
        assert request.headers.get("Cookie") == "lightcone-token=explicit-token"
        assert "Origin" not in request.headers
        socket = web.WebSocketResponse()
        await socket.prepare(request)
        async for message in socket:
            if message.type != WSMsgType.TEXT:
                continue
            incoming = message.json()
            if incoming.get("method") != "subscribe":
                continue
            subscriptions.append(incoming["params"])
            for data in [
                {
                    "status": "authenticated",
                    "wallet": "11111111111111111111111111111111",
                },
                {"status": "anonymous"},
                {"status": "anonymous", "reason": "TOKEN_EXPIRED"},
                {"status": "anonymous", "reason": "TOKEN_REVOKED"},
            ]:
                await socket.send_json({"type": "auth", "version": 0.1, "data": data})
            await socket.send_json(
                {
                    "type": "error",
                    "version": 0.1,
                    "data": {
                        "error": "retry snapshot",
                        "code": "PRIVATE_SNAPSHOT_UNAVAILABLE",
                        "wallet_address": "wallet-a",
                    },
                }
            )
            await socket.send_json({"type": "pong", "version": 0.1, "data": {}})
        return socket

    app = web.Application()
    app.router.add_get("/ws", handler)
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "127.0.0.1", 0)
    await site.start()
    port = site._server.sockets[0].getsockname()[1]
    client = WsClient(
        WsConfig(
            url=f"ws://127.0.0.1:{port}/ws", reconnect=True, ping_interval_ms=60_000
        )
    )
    client.set_auth_token("explicit-token")
    messages = []
    transport = []
    completed = asyncio.Event()

    def receive(event):
        if event.type == WsEventType.MESSAGE:
            messages.append(event.message)
            if event.message.type == MessageInType.PONG:
                completed.set()
        else:
            transport.append(event.type)

    client.on(receive)
    public = MarketParams(market_pubkey="11111111111111111111111111111111")
    try:
        await client.connect()
        await client.subscribe(public)
        await asyncio.wait_for(completed.wait(), 5)
        await asyncio.sleep(0.75)
        reasons = [
            message.data.reason
            for message in messages
            if message.type == MessageInType.AUTH and message.data.status == "anonymous"
        ]
        assert reasons == [None, "TOKEN_EXPIRED", "TOKEN_REVOKED"]
        error = next(
            message.data for message in messages if message.type == MessageInType.ERROR
        )
        assert error.code == "PRIVATE_SNAPSHOT_UNAVAILABLE"
        assert error.wallet_address == "wallet-a"
        assert client.is_connected()
        assert client._active_subscriptions == [public]
        assert connections == 1
        assert len(subscriptions) == 1
        assert transport == [WsEventType.CONNECTED]
    finally:
        await client.disconnect()
        await runner.cleanup()


@pytest.mark.asyncio
async def test_explicit_restart_recovers_after_handshake_retry_exhaustion():
    available = False
    attempts = 0
    subscribed = asyncio.Event()
    exhausted = asyncio.Event()

    async def handler(request):
        nonlocal attempts
        attempts += 1
        if not available:
            return web.Response(status=503)
        assert request.headers.get("Cookie") == "lightcone-token=retained-token"
        socket = web.WebSocketResponse()
        await socket.prepare(request)
        async for message in socket:
            if message.type == WSMsgType.TEXT:
                incoming = message.json()
                if incoming.get("method") == "subscribe":
                    assert incoming["params"]["type"] == "market"
                    subscribed.set()
        return socket

    app = web.Application()
    app.router.add_get("/ws", handler)
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "127.0.0.1", 0)
    await site.start()
    port = site._server.sockets[0].getsockname()[1]
    client = WsClient(
        WsConfig(
            url=f"ws://127.0.0.1:{port}/ws",
            reconnect=True,
            max_reconnect_attempts=0,
            ping_interval_ms=60_000,
        )
    )
    client.set_auth_token("retained-token")

    def receive(event):
        if event.type == WsEventType.MAX_RECONNECT_REACHED:
            exhausted.set()

    client.on(receive)
    try:
        await client.connect()
        await asyncio.wait_for(exhausted.wait(), 5)
        assert attempts == 1
        available = True
        await client.restart_connection()
        await client.subscribe(
            MarketParams(market_pubkey="11111111111111111111111111111111")
        )
        await asyncio.wait_for(subscribed.wait(), 5)
        assert client.is_connected()
        assert attempts == 2
    finally:
        await client.disconnect()
        await runner.cleanup()
