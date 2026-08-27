"""WebSocket subscription management."""

from dataclasses import dataclass, field
from typing import Optional, Union

from ..domain.orderbook.aggregation import BookAggregation


# ---------------------------------------------------------------------------
# Subscribe/Unsubscribe parameter types
# ---------------------------------------------------------------------------


@dataclass
class BookUpdateParams:
    """Book snapshots, optionally aggregated (Hyperliquid-style).

    Each ``(orderbook, aggregation)`` pair is a distinct subscription — one
    connection may hold multiple aggregation views of the same orderbook, and
    unsubscribe must repeat the same (normalized) aggregation to match. On the
    wire ``n_sig_figs`` is spelled ``nSigFigs`` and ``None`` fields are
    omitted (the backend rejects unknown/null params); the client handles
    that mapping when sending.
    """

    type: str = "book_update"
    orderbook_ids: list[str] = field(default_factory=list)
    n_sig_figs: Optional[int] = None
    mantissa: Optional[int] = None


@dataclass
class TradesParams:
    type: str = "trades"
    orderbook_ids: list[str] = field(default_factory=list)


@dataclass
class UserParams:
    """Authenticated user stream keyed by the exact wallet identity."""

    type: str = "user"
    wallet_address: str = ""


@dataclass
class WalletDepositBalancesParams:
    """Authenticated wallet balance stream tracked for reconnect replay."""

    type: str = "wallet_deposit_balances"
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


# Closed set of locally tracked subscription identities.
SubscribeParams = Union[
    BookUpdateParams,
    TradesParams,
    UserParams,
    WalletDepositBalancesParams,
    PriceHistoryParams,
    TickerParams,
    MarketParams,
    DepositPriceParams,
    DepositAssetPriceParams,
]

# Wire unsubscribe identities deliberately reuse the corresponding subscribe shape.
UnsubscribeParams = SubscribeParams


# ---------------------------------------------------------------------------
# Subscription key & matching
# ---------------------------------------------------------------------------


def subscription_key(params: SubscribeParams) -> str:
    """Return the stable deduplication and reconnect-replay identity."""
    if isinstance(params, BookUpdateParams):
        ids = ",".join(sorted(params.orderbook_ids))
        aggregation = BookAggregation.from_frame(params.n_sig_figs, params.mantissa)
        # Full precision keeps the pre-aggregation key shape so existing
        # consumers' tracked subscriptions stay stable. Normalization makes
        # (5, None) and (5, 1) the same subscription.
        if aggregation.is_full():
            return f"book:{ids}"
        return f"book:{ids}:{aggregation.key_suffix()}"
    elif isinstance(params, TradesParams):
        ids = ",".join(sorted(params.orderbook_ids))
        return f"trades:{ids}"
    elif isinstance(params, UserParams):
        return f"user:{params.wallet_address}"
    elif isinstance(params, WalletDepositBalancesParams):
        return f"wallet_deposit_balances:{params.wallet_address}"
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
    """Check whether a wire unsubscribe removes this tracked replay identity."""
    if type(sub) != type(unsub):
        return False
    return subscription_key(sub) == subscription_key(unsub)


__all__ = [
    "BookUpdateParams",
    "TradesParams",
    "UserParams",
    "WalletDepositBalancesParams",
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
