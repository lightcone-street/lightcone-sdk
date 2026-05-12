"""Trades sub-client."""

from __future__ import annotations

from typing import Optional, TYPE_CHECKING

from . import Trade, TradesPage
from .wire import MarketTradesResponseWire, TradesResponseWire
from .convert import trade_from_wire

if TYPE_CHECKING:
    from ...client import LightconeClient


class Trades:
    """Trade operations sub-client."""

    def __init__(self, client: "LightconeClient"):
        self._client = client

    async def get(
        self,
        orderbook_id: str,
        limit: Optional[int] = None,
        cursor: Optional[int] = None,
    ) -> TradesPage:
        """Get trades for an orderbook."""
        url = f"/api/trades?orderbook_id={orderbook_id}"
        if limit is not None:
            url += f"&limit={limit}"
        if cursor is not None:
            url += f"&cursor={cursor}"

        data = await self._client._http.get(url)
        resp = TradesResponseWire.from_dict(data)
        trades = [trade_from_wire(t) for t in resp.trades]
        return TradesPage(
            trades=trades,
            next_cursor=resp.next_cursor,
            has_more=resp.has_more,
        )

    async def get_by_market(
        self,
        market_pubkey: str,
        limit: Optional[int] = None,
        cursor: Optional[int] = None,
    ) -> TradesPage:
        """Get trades for all orderbooks in a market, interleaved by time."""
        url = f"/api/trades/market?market_pubkey={market_pubkey}"
        if limit is not None:
            url += f"&limit={limit}"
        if cursor is not None:
            url += f"&cursor={cursor}"

        data = await self._client._http.get(url)
        resp = MarketTradesResponseWire.from_dict(data)
        trades = [trade_from_wire(t) for t in resp.trades]
        return TradesPage(
            trades=trades,
            next_cursor=resp.next_cursor,
            has_more=resp.has_more,
        )
