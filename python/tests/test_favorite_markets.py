"""Favorite markets client contract tests."""

import pytest
from aiohttp import web

from lightcone_sdk.client import LightconeClientBuilder


@pytest.mark.asyncio
async def test_favorite_markets_methods_use_contract_verbs_paths_and_cookies() -> None:
    requests: list[tuple[str, str, str | None]] = []

    async def handler(request: web.Request) -> web.Response:
        requests.append((request.method, request.raw_path, request.headers.get("Cookie")))
        if request.method == "GET":
            body = {"market_pubkeys": ["market-a"]}
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
        assert await client.markets().favorite_markets_with_cookies("lightcone-token=test") == ["market-a"]
        assert (await client.markets().add_favorite_market_with_cookies("market/one", "lightcone-token=test"))["favorited"] is True
        assert (await client.markets().remove_favorite_market_with_cookies("market/one", "lightcone-token=test"))["favorited"] is False
        assert requests == [
            ("GET", "/api/users/favorite-markets", "lightcone-token=test"),
            ("POST", "/api/users/favorite-markets/market%2Fone", "lightcone-token=test"),
            ("DELETE", "/api/users/favorite-markets/market%2Fone", "lightcone-token=test"),
        ]
    finally:
        await client.close()
        await runner.cleanup()
