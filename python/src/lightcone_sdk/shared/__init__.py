"""Shared utilities used across the Lightcone SDK."""

from .api_response import ApiRejectedDetails, ApiResponse
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
from .price import format_decimal, is_zero, parse_decimal
from .rejection import RejectionCode
from .scaling import (
    I64_MAX,
    PRICE_SCALE,
    U32_MAX,
    OrderbookRules,
    ScaledAmounts,
    ScalingError,
    TradingRules,
    exact_scaled_integer,
    scale_price_size,
    validate_raw_amounts,
    validate_signed_fields,
)
from .signing import (
    ExternalSigner,
    SigningStrategy,
    SigningStrategyKind,
    classify_signer_error,
)
from .types import (
    Denominator,
    DepositSource,
    OrderBookId,
    OrderUpdateType,
    PubkeyStr,
    Resolution,
    Side,
    SubmitOrderRequest,
    TimeInForce,
)


def derive_orderbook_id(base_token: str, quote_token: str) -> str:
    """Derive an orderbook ID from base and quote token pubkeys.

    Format: "{base[0:8]}_{quote[0:8]}"
    """
    return f"{base_token[:8]}_{quote_token[:8]}"


__all__ = [
    # Types
    "OrderBookId",
    "PubkeyStr",
    "ApiResponse",
    "ApiRejectedDetails",
    "RejectionCode",
    "Side",
    "Denominator",
    "TimeInForce",
    "OrderUpdateType",
    "DepositSource",
    "Resolution",
    "SubmitOrderRequest",
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
    "OrderbookRules",
    "TradingRules",
    "ScaledAmounts",
    "ScalingError",
    "exact_scaled_integer",
    "scale_price_size",
    "validate_raw_amounts",
    "validate_signed_fields",
    "PRICE_SCALE",
    "I64_MAX",
    "U32_MAX",
    # Signing
    "ExternalSigner",
    "SigningStrategy",
    "SigningStrategyKind",
    "classify_signer_error",
    # Utils
    "derive_orderbook_id",
]
