"""Favorite markets client contract tests."""

import pytest
from aiohttp import web

from lightcone_sdk.client import LightconeClientBuilder


@pytest.mark.asyncio
async def test_favorite_markets_methods_use_contract_requests_and_retry_mutations() -> None:
    requests: list[tuple[str, str, str | None]] = []
    attempts: dict[tuple[str, str, str | None], int] = {}

    async def handler(request: web.Request) -> web.Response:
        requests.append((request.method, request.raw_path, request.headers.get("Cookie")))
        request_key = (request.method, request.raw_path, request.headers.get("Cookie"))
        attempts[request_key] = attempts.get(request_key, 0) + 1
        if request.method != "GET" and attempts[request_key] == 1:
            return web.json_response(
                {"status": "error", "error_details": {"reason": "retry"}}, status=503
            )
        if request.method == "GET":
            body = {"market_pubkeys": ["market-a"], "next_cursor": 12, "has_more": True}
        else:
            body = {"market_pubkey": "market/one", "favorited": request.method == "POST"}
        return web.json_response({"status": "success", "body": body})

    app = web.Application()
    app.router.add_route("*", "/{tail:.*}", handler)
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "127.0.0.1", 0)
    await site.start()
    sockets = site._server.sockets  # type: ignore[union-attr]
    client = LightconeClientBuilder().base_url(f"http://127.0.0.1:{sockets[0].getsockname()[1]}").build()

    try:
        favorites = await client.markets().favorite_markets_with_cookies(50, 7, "lightcone-token=test")
        assert favorites.market_pubkeys == ["market-a"]
        assert favorites.next_cursor == 12
        assert favorites.has_more is True
        assert (await client.markets().add_favorite_market("market/one")).favorited is True
        assert (await client.markets().add_favorite_market_with_cookies("market/one", "lightcone-token=test")).favorited is True
        assert (await client.markets().remove_favorite_market("market/one")).favorited is False
        assert (await client.markets().remove_favorite_market_with_cookies("market/one", "lightcone-token=test")).favorited is False
        assert requests == [
            ("GET", "/api/users/favorite-markets?limit=50&cursor=7", "lightcone-token=test"),
            ("POST", "/api/users/favorite-markets/market%2Fone", None),
            ("POST", "/api/users/favorite-markets/market%2Fone", None),
            ("POST", "/api/users/favorite-markets/market%2Fone", "lightcone-token=test"),
            ("POST", "/api/users/favorite-markets/market%2Fone", "lightcone-token=test"),
            ("DELETE", "/api/users/favorite-markets/market%2Fone", None),
            ("DELETE", "/api/users/favorite-markets/market%2Fone", None),
            ("DELETE", "/api/users/favorite-markets/market%2Fone", "lightcone-token=test"),
            ("DELETE", "/api/users/favorite-markets/market%2Fone", "lightcone-token=test"),
        ]
    finally:
        await client.close()
        await runner.cleanup()
