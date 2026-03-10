"""Shared utilities used across the Lightcone SDK."""

from .types import (
    OrderBookId,
    PubkeyStr,
    Side,
    TimeInForce,
    TriggerType,
    TriggerStatus,
    TriggerResultStatus,
    OrderUpdateType,
    TriggerUpdateType,
    Resolution,
    SubmitOrderRequest,
    SubmitTriggerOrderRequest,
)
from .price import parse_decimal, format_decimal, is_zero
from .scaling import OrderbookDecimals, ScaledAmounts, ScalingError, scale_price_size


def derive_orderbook_id(base_token: str, quote_token: str) -> str:
    """Derive an orderbook ID from base and quote token pubkeys.

    Format: "{base[0:8]}_{quote[0:8]}"
    """
    return f"{base_token[:8]}_{quote_token[:8]}"


__all__ = [
    # Types
    "OrderBookId",
    "PubkeyStr",
    "Side",
    "TimeInForce",
    "TriggerType",
    "TriggerStatus",
    "TriggerResultStatus",
    "OrderUpdateType",
    "TriggerUpdateType",
    "Resolution",
    "SubmitOrderRequest",
    "SubmitTriggerOrderRequest",
    # Price
    "parse_decimal",
    "format_decimal",
    "is_zero",
    # Scaling
    "OrderbookDecimals",
    "ScaledAmounts",
    "ScalingError",
    "scale_price_size",
    # Utils
    "derive_orderbook_id",
]
