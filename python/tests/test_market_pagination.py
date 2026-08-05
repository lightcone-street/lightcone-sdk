"""Markets pagination contract tests."""

import pytest
from aiohttp import web

from lightcone_sdk.client import LightconeClientBuilder


@pytest.mark.asyncio
async def test_markets_get_preserves_backend_pagination_metadata() -> None:
    requested_paths: list[str] = []

    async def handler(request: web.Request) -> web.Response:
        requested_paths.append(request.raw_path)
        return web.json_response(
            {
                "status": "success",
                "body": {
                    "markets": [],
                    "next_cursor": 42,
                    "has_more": True,
                },
            }
        )

    app = web.Application()
    app.router.add_get("/api/markets", handler)
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "127.0.0.1", 0)
    await site.start()
    sockets = site._server.sockets  # type: ignore[union-attr]
    client = (
        LightconeClientBuilder()
        .base_url(f"http://127.0.0.1:{sockets[0].getsockname()[1]}")
        .build()
    )

    try:
        page = await client.markets().get(cursor=7, limit=50)

        assert page.markets == []
        assert page.validation_errors == []
        assert page.next_cursor == 42
        assert page.has_more is True
        assert requested_paths == ["/api/markets?cursor=7&limit=50"]
    finally:
        await client.close()
        await runner.cleanup()
