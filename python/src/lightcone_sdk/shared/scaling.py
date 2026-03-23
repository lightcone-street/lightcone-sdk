"""Price and size scaling utilities for the Lightcone SDK."""

from dataclasses import dataclass
from decimal import Decimal, InvalidOperation, ROUND_DOWN


class ScalingError(Exception):
    """Error during price/size scaling."""

    pass


@dataclass
class OrderbookDecimals:
    """Decimal metadata for an orderbook pair."""

    orderbook_id: str
    base_decimals: int
    quote_decimals: int
    price_decimals: int
    tick_size: int = 0


@dataclass
class ScaledAmounts:
    """Result of scaling a price and size to raw lamport amounts."""

    amount_in: int
    amount_out: int


def align_price_to_tick(price: Decimal, decimals: OrderbookDecimals) -> Decimal:
    """Snap a price to the nearest valid tick.

    Converts to quote-token lamports, truncates to the nearest tick_size
    multiple, and converts back. Returns unchanged if tick_size is 0 or 1.
    """
    if decimals.tick_size <= 1:
        return price
    quote_multiplier = Decimal(10) ** decimals.quote_decimals
    tick = Decimal(decimals.tick_size)
    lamports = (price * quote_multiplier).to_integral_value()
    aligned = (lamports / tick).to_integral_value() * tick
    return aligned / quote_multiplier


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
        ScaledAmounts with amount_in and amount_out in raw lamports

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

    # Truncate size to base_decimals precision (strip f64 noise)
    size_d = size_d.quantize(Decimal(10) ** -decimals.base_decimals, rounding=ROUND_DOWN)

    # base_lamports = size * 10^base_decimals
    base_lamports = size_d * base_factor

    # Validate no fractional lamports
    if base_lamports != base_lamports.to_integral_value():
        raise ScalingError(f"Fractional lamports not allowed: base_lamports = {base_lamports}")

    # quote_lamports = size * price * 10^quote_decimals (truncate sub-lamport dust)
    quote_lamports = (size_d * price_d * quote_factor).to_integral_value(rounding=ROUND_DOWN)

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
            amount_in=quote_lamports_int,
            amount_out=base_lamports_int,
        )
    elif side == 1:  # ASK
        return ScaledAmounts(
            amount_in=base_lamports_int,
            amount_out=quote_lamports_int,
        )
    else:
        raise ScalingError(f"Invalid side: {side} (must be 0=BID or 1=ASK)")
