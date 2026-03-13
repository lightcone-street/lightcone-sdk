"""Trades sub-client."""

from typing import Optional, TYPE_CHECKING

from . import Trade, TradesPage
from .wire import TradesResponseWire
from .convert import trade_from_wire

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class Trades:
    """Trade operations sub-client."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def get(
        self,
        orderbook_id: str,
        limit: Optional[int] = None,
        before: Optional[str] = None,
    ) -> TradesPage:
        """Get trades for an orderbook."""
        params: dict = {"orderbook_id": orderbook_id}
        if limit is not None:
            params["limit"] = str(limit)
        if before is not None:
            params["before"] = before

        data = await self._http.get("/api/trades", params=params)
        resp = TradesResponseWire.from_dict(data)
        trades = [trade_from_wire(t) for t in resp.trades]
        return TradesPage(
            trades=trades,
            next_cursor=resp.next_cursor,
            has_more=resp.has_more,
        )
