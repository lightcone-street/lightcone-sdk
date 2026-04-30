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
    OrderbookTickersResponse,
    OrderbookVolumeMetrics,
    PlatformMetrics,
    UserMetrics,
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

    async def orderbook_tickers(
        self, deposit_asset: Optional[str] = None
    ) -> OrderbookTickersResponse:
        """GET /api/metrics/orderbooks/tickers[?deposit_asset=<mint>]

        Batch BBO + midpoint per active orderbook (same shape as the WS
        ``Ticker`` stream, delivered in one REST call). Optionally filter to
        orderbooks whose base conditional-token is backed by
        ``deposit_asset``. Prices per orderbook are scaled using that
        orderbook's own decimals.
        """
        url = "/api/metrics/orderbooks/tickers"
        mint = deposit_asset.strip() if deposit_asset else None
        if mint:
            url += f"?deposit_asset={url_quote(mint, safe='')}"
        data = await self._client._http.get(url)
        return OrderbookTickersResponse.from_dict(data)

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

    async def user(self) -> UserMetrics:
        """Per-wallet trading + referral aggregates for the authenticated user.

        Wallet is resolved server-side from the ``auth_token`` cookie.

        GET /api/metrics/user
        """
        data = await self._client._http.get("/api/metrics/user")
        return UserMetrics.from_dict(data)

    async def user_with_auth(self, auth_token: str) -> UserMetrics:
        """Same as :meth:`user`, with an explicit per-call ``auth_token``.

        For server-side cookie forwarding (SSR / route handlers).
        """
        data = await self._client._http.get_with_auth(
            "/api/metrics/user",
            auth_token=auth_token,
        )
        return UserMetrics.from_dict(data)

    async def user_by_wallet(self, wallet_address: str) -> UserMetrics:
        """Public variant of :meth:`user`.

        Takes the user's wallet via the URL path
        (``GET /api/metrics/user/{wallet_address}``) and requires no auth.
        """
        data = await self._client._http.get(
            f"/api/metrics/user/{url_quote(wallet_address, safe='')}"
        )
        return UserMetrics.from_dict(data)
