"""Shared utilities used across API and WebSocket modules.

Program-specific code has been moved to the program module.
"""

from .types import Resolution
from .price import parse_decimal, format_decimal
from .scaling import OrderbookDecimals, ScaledAmounts, ScalingError, scale_price_size


def derive_orderbook_id(base_token: str, quote_token: str) -> str:
    """Derive an orderbook ID from base and quote token pubkeys.

    Format: "{base[0:8]}_{quote[0:8]}"
    """
    return f"{base_token[:8]}_{quote_token[:8]}"


__all__ = [
    "Resolution",
    "parse_decimal",
    "format_decimal",
    "derive_orderbook_id",
    "OrderbookDecimals",
    "ScaledAmounts",
    "ScalingError",
    "scale_price_size",
]
