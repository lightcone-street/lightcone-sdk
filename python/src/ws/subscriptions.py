"""WebSocket subscription management."""

from dataclasses import dataclass
from typing import Optional, Union


# ---------------------------------------------------------------------------
# Subscribe/Unsubscribe parameter types
# ---------------------------------------------------------------------------


@dataclass
class BookUpdateParams:
    type: str = "book_update"
    orderbook_ids: list[str] = None  # type: ignore

    def __post_init__(self):
        if self.orderbook_ids is None:
            self.orderbook_ids = []


@dataclass
class TradesParams:
    type: str = "trades"
    orderbook_ids: list[str] = None  # type: ignore

    def __post_init__(self):
        if self.orderbook_ids is None:
            self.orderbook_ids = []


@dataclass
class UserParams:
    type: str = "user"
    wallet_address: str = ""


@dataclass
class PriceHistoryParams:
    type: str = "price_history"
    orderbook_id: str = ""
    resolution: str = "1m"
    include_ohlcv: bool = False


@dataclass
class TickerParams:
    type: str = "ticker"
    orderbook_ids: list[str] = None  # type: ignore

    def __post_init__(self):
        if self.orderbook_ids is None:
            self.orderbook_ids = []


@dataclass
class MarketParams:
    type: str = "market"
    market_pubkey: str = ""


SubscribeParams = Union[
    BookUpdateParams,
    TradesParams,
    UserParams,
    PriceHistoryParams,
    TickerParams,
    MarketParams,
]

UnsubscribeParams = SubscribeParams  # Same shapes for unsubscribe


# ---------------------------------------------------------------------------
# Subscription key & matching
# ---------------------------------------------------------------------------


def subscription_key(params: SubscribeParams) -> str:
    """Generate a unique key for a subscription for deduplication."""
    if isinstance(params, BookUpdateParams):
        ids = ",".join(sorted(params.orderbook_ids))
        return f"book:{ids}"
    elif isinstance(params, TradesParams):
        ids = ",".join(sorted(params.orderbook_ids))
        return f"trades:{ids}"
    elif isinstance(params, UserParams):
        return f"user:{params.wallet_address}"
    elif isinstance(params, PriceHistoryParams):
        return f"price_history:{params.orderbook_id}:{params.resolution}"
    elif isinstance(params, TickerParams):
        ids = ",".join(sorted(params.orderbook_ids))
        return f"ticker:{ids}"
    elif isinstance(params, MarketParams):
        return f"market:{params.market_pubkey}"
    return f"unknown:{id(params)}"


def unsubscribe_matches(sub: SubscribeParams, unsub: UnsubscribeParams) -> bool:
    """Check if an unsubscribe matches a subscribe."""
    if type(sub) != type(unsub):
        return False
    return subscription_key(sub) == subscription_key(unsub)


__all__ = [
    "BookUpdateParams",
    "TradesParams",
    "UserParams",
    "PriceHistoryParams",
    "TickerParams",
    "MarketParams",
    "SubscribeParams",
    "UnsubscribeParams",
    "subscription_key",
    "unsubscribe_matches",
]
