"""Metrics sub-client — platform / market / orderbook / category / deposit-token
volume metrics, market leaderboard, and time-series history."""

from __future__ import annotations

from collections.abc import Awaitable, Callable
from typing import TYPE_CHECKING, Optional
from urllib.parse import quote as url_quote

from ...error import ValidationError
from ...shared.query import query_u32

from .wire import (
    CategoriesMetrics,
    CategoryVolumeMetrics,
    DepositTokenVolumeHistory,
    DepositTokenVolumeHistoryQuery,
    DepositTokensMetrics,
    Leaderboard,
    MarketDetailMetrics,
    MarketsMetrics,
    MetricsHistory,
    MetricsHistoryQuery,
    OpenInterestHistory,
    OpenInterestHistoryQuery,
    OrderbookTickersResponse,
    OrderbookVolumeMetrics,
    PlatformMetrics,
    UniqueTradersHistory,
    UniqueTradersHistoryQuery,
    UserMetrics,
)

if TYPE_CHECKING:
    from ...client import LightconeClient

#: Upper bound on pages followed by :meth:`Metrics.orderbook_tickers` (8
#: tickers per page), so a misbehaving cursor can never loop forever.
MAX_TICKER_PAGES = 1_000


async def _collect_ticker_pages(
    max_pages: int,
    fetch: Callable[[str | None], Awaitable[OrderbookTickersResponse]],
) -> OrderbookTickersResponse:
    """Follow ticker pages until the last one or ``max_pages``, which leaves
    ``has_more`` true with the cursor to resume from. A page that reports more
    without an advancing cursor raises ``ValidationError`` instead of
    truncating silently."""
    tickers = []
    cursor: str | None = None
    for _ in range(max_pages):
        response = await fetch(cursor)
        tickers.extend(response.tickers)
        if not response.has_more:
            return OrderbookTickersResponse(tickers=tickers)
        next_cursor = response.next_cursor
        if next_cursor is None or next_cursor == cursor:
            raise ValidationError(
                "orderbook tickers page reported more without an advancing cursor "
                f"(after {cursor}: {next_cursor})"
            )
        cursor = next_cursor
    return OrderbookTickersResponse(tickers=tickers, next_cursor=cursor, has_more=True)


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
        """GET /api/metrics/orderbooks/tickers[?deposit_asset=<mint>&cursor=<orderbook>]

        Batch BBO + midpoint for every ready orderbook (same shape as the WS
        ``Ticker`` stream). Optionally filter to orderbooks whose base
        conditional-token is backed by ``deposit_asset``. Prices per orderbook
        are scaled using that orderbook's own decimals.

        The endpoint pages at most 8 tickers per request; this follows every
        page and returns them merged with ``has_more`` false. If it stops at
        its page cap first, ``has_more`` stays true and ``next_cursor``
        resumes with :meth:`orderbook_tickers_page`. A page that reports more
        without an advancing cursor raises ``ValidationError``.
        """
        return await _collect_ticker_pages(
            MAX_TICKER_PAGES,
            lambda cursor: self.orderbook_tickers_page(deposit_asset, cursor),
        )

    async def orderbook_tickers_page(
        self,
        deposit_asset: str | None = None,
        cursor: str | None = None,
        limit: int | None = None,
    ) -> OrderbookTickersResponse:
        """One page of :meth:`orderbook_tickers`.

        ``cursor`` is a previous page's ``next_cursor`` (an orderbook pubkey);
        ``limit`` defaults to 8 and must be within 1..8 (other values are
        rejected with ``INVALID_LIMIT``).
        """
        params: dict[str, str] = {}
        mint = deposit_asset.strip() if deposit_asset else None
        if mint:
            params["deposit_asset"] = mint
        if cursor is not None:
            params["cursor"] = cursor
        if limit is not None:
            params["limit"] = query_u32("limit", limit)
        data = await self._client._http.get(
            "/api/metrics/orderbooks/tickers", params=params or None
        )
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

    async def deposit_tokens_volume_history(
        self, query: Optional[DepositTokenVolumeHistoryQuery] = None
    ) -> DepositTokenVolumeHistory:
        """GET /api/metrics/deposit-tokens/volume-history"""
        params = (query or DepositTokenVolumeHistoryQuery()).to_query()
        data = await self._client._http.get(
            "/api/metrics/deposit-tokens/volume-history",
            params=params or None,
        )
        return DepositTokenVolumeHistory.from_dict(data)

    async def open_interest_history(
        self, query: Optional[OpenInterestHistoryQuery] = None
    ) -> OpenInterestHistory:
        """GET /api/metrics/open-interest/history"""
        params = (query or OpenInterestHistoryQuery()).to_query()
        data = await self._client._http.get(
            "/api/metrics/open-interest/history",
            params=params or None,
        )
        return OpenInterestHistory.from_dict(data)

    async def unique_traders_history(
        self, query: Optional[UniqueTradersHistoryQuery] = None
    ) -> UniqueTradersHistory:
        """GET /api/metrics/unique-traders/history"""
        params = (query or UniqueTradersHistoryQuery()).to_query()
        data = await self._client._http.get(
            "/api/metrics/unique-traders/history",
            params=params or None,
        )
        return UniqueTradersHistory.from_dict(data)

    async def leaderboard(self, limit: Optional[int] = None) -> Leaderboard:
        """GET /api/metrics/leaderboard/markets"""
        params: dict[str, str] | None = None
        if limit is not None:
            params = {"limit": str(limit)}
        data = await self._client._http.get(
            "/api/metrics/leaderboard/markets", params=params
        )
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
        path = (
            f"/api/metrics/history/"
            f"{url_quote(scope, safe='')}/{url_quote(scope_key, safe='')}"
        )
        params = (query or MetricsHistoryQuery()).to_query()
        data = await self._client._http.get(path, params=params or None)
        return MetricsHistory.from_dict(data)

    async def user(self) -> UserMetrics:
        """Per-wallet trading + referral aggregates for the authenticated user.

        Wallet is resolved server-side from the ``cookie_header`` cookie.

        GET /api/metrics/user
        """
        data = await self._client._http.get("/api/metrics/user")
        return UserMetrics.from_dict(data)

    async def user_with_cookies(self, cookie_header: str) -> UserMetrics:
        """Same as :meth:`user`, with an explicit per-call ``cookie_header``.

        For server-side cookie forwarding (SSR / route handlers).
        """
        data = await self._client._http.get_with_cookies(
            "/api/metrics/user",
            cookie_header=cookie_header,
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
