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
    DepositSource,
    Resolution,
    SubmitOrderRequest,
    SubmitTriggerOrderRequest,
)
from .fmt import (
    abbr_number,
    display,
    display_decimal,
    display_formatted_string,
    display_with_decimals,
    from_decimal_value,
    to_base_units,
    to_decimal_value,
)
from .price import parse_decimal, format_decimal, is_zero
from .scaling import OrderbookDecimals, ScaledAmounts, ScalingError, align_price_to_tick, scale_price_size
from .signing import ExternalSigner, SigningStrategy, SigningStrategyKind, classify_signer_error


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
    "DepositSource",
    "Resolution",
    "SubmitOrderRequest",
    "SubmitTriggerOrderRequest",
    # Formatting
    "abbr_number",
    "display",
    "display_decimal",
    "display_formatted_string",
    "display_with_decimals",
    "from_decimal_value",
    "to_base_units",
    "to_decimal_value",
    # Price
    "parse_decimal",
    "format_decimal",
    "is_zero",
    # Scaling
    "OrderbookDecimals",
    "ScaledAmounts",
    "ScalingError",
    "align_price_to_tick",
    "scale_price_size",
    # Signing
    "ExternalSigner",
    "SigningStrategy",
    "SigningStrategyKind",
    "classify_signer_error",
    # Utils
    "derive_orderbook_id",
]
