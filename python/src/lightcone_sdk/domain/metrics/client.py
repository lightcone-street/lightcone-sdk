"""Metrics sub-client — platform / market / orderbook / category / deposit-token
volume metrics, market leaderboard, and time-series history."""

from __future__ import annotations

from typing import TYPE_CHECKING, Optional
from urllib.parse import quote as url_quote, urlencode

from .wire import (
    CategoriesMetrics,
    CategoryVolumeMetrics,
    DepositTokensMetrics,
    Leaderboard,
    MarketDetailMetrics,
    MarketsMetrics,
    MetricsHistory,
    MetricsHistoryQuery,
    OrderbookVolumeMetrics,
    PlatformMetrics,
)

if TYPE_CHECKING:
    from ...client import LightconeClient


class Metrics:
    """Metrics sub-client. Obtain via ``client.metrics()``."""

    def __init__(self, client: "LightconeClient"):
        self._client = client

    async def platform(self) -> PlatformMetrics:
        """GET /api/metrics/platform"""
        data = await self._client._http.get("/api/metrics/platform")
        return PlatformMetrics.from_dict(data)

    async def markets(self) -> MarketsMetrics:
        """GET /api/metrics/markets"""
        data = await self._client._http.get("/api/metrics/markets")
        return MarketsMetrics.from_dict(data)

    async def market(self, market_pubkey: str) -> MarketDetailMetrics:
        """GET /api/metrics/markets/{market_pubkey}"""
        data = await self._client._http.get(
            f"/api/metrics/markets/{url_quote(market_pubkey, safe='')}"
        )
        return MarketDetailMetrics.from_dict(data)

    async def orderbook(self, orderbook_id: str) -> OrderbookVolumeMetrics:
        """GET /api/metrics/orderbooks/{orderbook_id}"""
        data = await self._client._http.get(
            f"/api/metrics/orderbooks/{url_quote(orderbook_id, safe='')}"
        )
        return OrderbookVolumeMetrics.from_dict(data)

    async def categories(self) -> CategoriesMetrics:
        """GET /api/metrics/categories"""
        data = await self._client._http.get("/api/metrics/categories")
        return CategoriesMetrics.from_dict(data)

    async def category(self, category: str) -> CategoryVolumeMetrics:
        """GET /api/metrics/categories/{category}"""
        data = await self._client._http.get(
            f"/api/metrics/categories/{url_quote(category, safe='')}"
        )
        return CategoryVolumeMetrics.from_dict(data)

    async def deposit_tokens(self) -> DepositTokensMetrics:
        """GET /api/metrics/deposit-tokens"""
        data = await self._client._http.get("/api/metrics/deposit-tokens")
        return DepositTokensMetrics.from_dict(data)

    async def leaderboard(self, limit: Optional[int] = None) -> Leaderboard:
        """GET /api/metrics/leaderboard/markets"""
        url = "/api/metrics/leaderboard/markets"
        if limit is not None:
            url += f"?limit={limit}"
        data = await self._client._http.get(url)
        return Leaderboard.from_dict(data)

    async def history(
        self,
        scope: str,
        scope_key: str,
        query: Optional[MetricsHistoryQuery] = None,
    ) -> MetricsHistory:
        """GET /api/metrics/history/{scope}/{scope_key}

        ``scope`` is one of ``"orderbook" | "market" | "category" |
        "deposit_token" | "platform"``.
        """
        url = (
            f"/api/metrics/history/"
            f"{url_quote(scope, safe='')}/{url_quote(scope_key, safe='')}"
        )
        params = (query or MetricsHistoryQuery()).to_query()
        if params:
            url += "?" + urlencode(params)
        data = await self._client._http.get(url)
        return MetricsHistory.from_dict(data)
