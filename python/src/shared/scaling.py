"""Price and size scaling utilities for the Lightcone SDK."""

from dataclasses import dataclass
from decimal import Decimal, InvalidOperation


class ScalingError(Exception):
    """Error during price/size scaling."""

    pass


@dataclass
class OrderbookDecimals:
    """Decimal metadata for an orderbook pair."""

    base_decimals: int
    quote_decimals: int
    price_decimals: int


@dataclass
class ScaledAmounts:
    """Result of scaling a price and size to raw lamport amounts."""

    maker_amount: int
    taker_amount: int


def scale_price_size(
    price: str,
    size: str,
    side: int,
    decimals: OrderbookDecimals,
) -> ScaledAmounts:
    """Scale a human-readable price and size to raw on-chain amounts.

    Args:
        price: Price as a decimal string (e.g., "0.55")
        size: Size as a decimal string (e.g., "100.0")
        side: 0 for BID, 1 for ASK
        decimals: Decimal configuration for the orderbook

    Returns:
        ScaledAmounts with maker_amount and taker_amount in raw lamports

    Raises:
        ScalingError: If inputs are invalid or result in overflow
    """
    try:
        price_d = Decimal(price)
        size_d = Decimal(size)
    except (InvalidOperation, ValueError) as e:
        raise ScalingError(f"Invalid decimal input: {e}")

    if price_d <= 0:
        raise ScalingError(f"Price must be positive, got {price}")
    if size_d <= 0:
        raise ScalingError(f"Size must be positive, got {size}")

    base_factor = Decimal(10) ** decimals.base_decimals
    quote_factor = Decimal(10) ** decimals.quote_decimals

    # base_lamports = size * 10^base_decimals
    base_lamports = size_d * base_factor
    # quote_lamports = size * price * 10^quote_decimals
    quote_lamports = size_d * price_d * quote_factor

    # Round to integers
    base_lamports_int = int(base_lamports)
    quote_lamports_int = int(quote_lamports)

    if base_lamports_int == 0:
        raise ScalingError("Computed base_lamports is zero (size too small)")
    if quote_lamports_int == 0:
        raise ScalingError("Computed quote_lamports is zero (price * size too small)")

    max_u64 = 2**64 - 1
    if base_lamports_int > max_u64:
        raise ScalingError(f"base_lamports overflow: {base_lamports_int}")
    if quote_lamports_int > max_u64:
        raise ScalingError(f"quote_lamports overflow: {quote_lamports_int}")

    # BID: maker gives quote, wants base
    # ASK: maker gives base, wants quote
    if side == 0:  # BID
        return ScaledAmounts(
            maker_amount=quote_lamports_int,
            taker_amount=base_lamports_int,
        )
    elif side == 1:  # ASK
        return ScaledAmounts(
            maker_amount=base_lamports_int,
            taker_amount=quote_lamports_int,
        )
    else:
        raise ScalingError(f"Invalid side: {side} (must be 0=BID or 1=ASK)")
