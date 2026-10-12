"""Trade domain types."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from decimal import Decimal

from ...shared.types import OrderBookId, Side


@dataclass(kw_only=True)
class Trade:
    """A trade execution record."""

    orderbook_id: OrderBookId
    #: ``"<execution_id>:<leg_index>:<projection_generation>"``, identical for
    #: REST and WS trades, so it deduplicates merged history.
    trade_id: str
    timestamp: datetime
    #: Price in quote units per base unit. REST reports the maker price
    #: truncated to the book's price decimals; WS trades derive the exact
    #: ``quote_amount / base_amount``.
    price: Decimal
    #: Size in base-token units.
    size: Decimal
    #: Taker side.
    side: Side
    #: Numeric REST row id (i64) used for cursor pagination. ``None`` on WS trades.
    cursor_id: int | None = None


@dataclass(kw_only=True)
class TradesPage:
    """A page of trades with cursor-based pagination metadata."""

    trades: list[Trade] = field(default_factory=list)
    next_cursor: int | None = None
    has_more: bool = False


from .convert import trade_from_wire, trade_from_ws  # noqa: E402
from .state import TradeHistory  # noqa: E402
from .wire import (  # noqa: E402
    MarketTradesResponseWire,
    TradeResponseWire,
    TradesDecimals,
    TradesResponseWire,
    WsTrade,
)

__all__ = [
    "MarketTradesResponseWire",
    "Trade",
    "TradeHistory",
    "TradeResponseWire",
    "TradesDecimals",
    "TradesPage",
    "TradesResponseWire",
    "WsTrade",
    "trade_from_wire",
    "trade_from_ws",
]
