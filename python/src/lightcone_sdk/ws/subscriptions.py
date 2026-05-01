"""WebSocket subscription management."""

from dataclasses import dataclass, field
from typing import Optional, Union


# ---------------------------------------------------------------------------
# Subscribe/Unsubscribe parameter types
# ---------------------------------------------------------------------------


@dataclass
class BookUpdateParams:
    type: str = "book_update"
    orderbook_ids: list[str] = field(default_factory=list)


@dataclass
class TradesParams:
    type: str = "trades"
    orderbook_ids: list[str] = field(default_factory=list)


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
    orderbook_ids: list[str] = field(default_factory=list)


@dataclass
class MarketParams:
    type: str = "market"
    market_pubkey: str = ""


@dataclass
class DepositPriceParams:
    type: str = "deposit_price"
    deposit_asset: str = ""
    resolution: str = "1m"


@dataclass
class DepositAssetPriceParams:
    """Subscribe to the live spot price for one deposit asset.

    Snapshot on subscribe + per-asset price ticks. Distinct from
    `DepositPriceParams` which carries OHLCV candles per resolution.
    """

    type: str = "deposit_asset_price"
    deposit_asset: str = ""


SubscribeParams = Union[
    BookUpdateParams,
    TradesParams,
    UserParams,
    PriceHistoryParams,
    TickerParams,
    MarketParams,
    DepositPriceParams,
    DepositAssetPriceParams,
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
    elif isinstance(params, DepositPriceParams):
        return f"deposit_price:{params.deposit_asset}:{params.resolution}"
    elif isinstance(params, DepositAssetPriceParams):
        return f"deposit_asset_price:{params.deposit_asset}"
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
    "DepositPriceParams",
    "DepositAssetPriceParams",
    "SubscribeParams",
    "UnsubscribeParams",
    "subscription_key",
    "unsubscribe_matches",
]
