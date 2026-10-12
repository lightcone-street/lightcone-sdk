"""Price utilities used across the Lightcone SDK."""

from __future__ import annotations

from decimal import Decimal


def parse_decimal(s: str) -> Decimal:
    """Parse a decimal string to ``Decimal``."""
    return Decimal(s)


def format_decimal(value: Decimal | float | str, precision: int = 6) -> str:
    """Format a value as a decimal string with the specified precision.

    The implementation is ``Decimal``-first to avoid float-driven rounding
    surprises in request payloads and typed domain conversions.
    """
    decimal_value = value if isinstance(value, Decimal) else Decimal(str(value))
    quantized = decimal_value.quantize(Decimal(1).scaleb(-precision))
    return format(quantized, "f")


def is_zero(value: str) -> bool:
    """Check if a decimal string represents zero."""
    try:
        return Decimal(value) == 0
    except Exception:
        return False


_MAX_MANTISSA = (1 << 96) - 1
_MAX_SCALE = 28


def _scaled(value: Decimal) -> tuple[int, int]:
    """Split a finite decimal into a signed integer mantissa and a scale >= 0."""
    if not value.is_finite():
        raise ValueError(f"expected a finite decimal, got {value}")
    sign, digits, exponent = value.as_tuple()
    mantissa = int("".join(map(str, digits)) or "0")
    if exponent > 0:
        mantissa *= 10**exponent
        exponent = 0
    return (-mantissa if sign else mantissa), -exponent


def _from_scaled(mantissa: int, scale: int) -> Decimal:
    """Build the exact decimal ``mantissa * 10**-scale`` (independent of context)."""
    digits = tuple(int(char) for char in str(abs(mantissa)))
    return Decimal((1 if mantissa < 0 else 0, digits, -scale))


def _div_round_half_even(numerator: int, denominator: int) -> int:
    quotient, remainder = divmod(numerator, denominator)
    twice = remainder * 2
    if twice > denominator or (twice == denominator and quotient % 2 == 1):
        quotient += 1
    return quotient


def quote_per_base(quote: Decimal, base: Decimal) -> Decimal | None:
    """Exact quote per base, normalized (no trailing zeros, no exponent form).

    Matches the Rust SDK's ``quote_per_base``: the quotient keeps the largest
    scale, at most 28, whose mantissa fits in 96 bits, and rounds half to
    even. Returns ``None`` for a zero base amount or a quotient beyond the
    96-bit range. Independent of the active ``decimal`` context.
    """
    quote_mantissa, quote_scale = _scaled(quote)
    base_mantissa, base_scale = _scaled(base)
    if base_mantissa == 0:
        return None
    negative = (quote_mantissa < 0) != (base_mantissa < 0)
    numerator = abs(quote_mantissa) * 10**base_scale
    denominator = abs(base_mantissa) * 10**quote_scale
    for scale in range(_MAX_SCALE, -1, -1):
        scaled = numerator * 10**scale
        if scaled // denominator > _MAX_MANTISSA:
            continue
        mantissa = _div_round_half_even(scaled, denominator)
        if mantissa > _MAX_MANTISSA:
            continue
        if mantissa == 0:
            return Decimal(0)
        while scale > 0 and mantissa % 10 == 0:
            mantissa //= 10
            scale -= 1
        return _from_scaled(-mantissa if negative else mantissa, scale)
    return None


def add_decimals(left: Decimal, right: Decimal) -> Decimal:
    """Exact sum at the larger of the two scales, as the Rust SDK decimal adds.

    Independent of the active ``decimal`` context, whose precision would
    otherwise round the sum.
    """
    left_mantissa, left_scale = _scaled(left)
    right_mantissa, right_scale = _scaled(right)
    scale = max(left_scale, right_scale)
    total = left_mantissa * 10 ** (scale - left_scale) + right_mantissa * 10 ** (
        scale - right_scale
    )
    return _from_scaled(total, scale)
