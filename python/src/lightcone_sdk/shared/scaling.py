"""Exact order construction and immutable trading-rule validation."""

from __future__ import annotations

import re
from dataclasses import dataclass
from decimal import Decimal
from typing import Union

PRICE_SCALE = 1_000_000
I64_MAX = 2**63 - 1
U32_MAX = 2**32 - 1
ExactDecimal = Union[str, Decimal]


class ScalingError(ValueError):
    """An order cannot be represented under the engine admission rules."""

    def __init__(self, message: str, code: str | None = None):
        super().__init__(message)
        self.code = code

    @classmethod
    def from_code(cls, code: str, detail: str | None = None) -> "ScalingError":
        return cls(f"{code}: {detail}" if detail else code, code=code)


@dataclass(frozen=True)
class TradingRules:
    base_size_decimals: int
    max_price_decimals: int
    max_price_significant_figures: int
    integer_prices_always_allowed: bool
    price_quantum: str
    price_quantum_raw: int
    base_size_quantum: str
    base_size_quantum_raw: int


@dataclass(frozen=True)
class OrderbookRules:
    orderbook_id: str
    base_decimals: int
    quote_decimals: int
    price_decimals: int
    trading_rules: TradingRules

    def validate_for_orderbook(self, orderbook_id: str) -> None:
        """Reject rules supplied for a different orderbook."""
        if self.orderbook_id != orderbook_id:
            raise ScalingError(
                f"trading rules for orderbook '{self.orderbook_id}' "
                f"cannot be used for '{orderbook_id}'"
            )


@dataclass(frozen=True)
class ScaledAmounts:
    amount_in: int
    amount_out: int
    price_raw: int
    base_atoms: int
    quote_atoms: int


_DECIMAL_RE = re.compile(
    r"^\+?(?:(?P<whole>\d+)(?:\.(?P<fraction>\d*))?|\.(?P<leading_fraction>\d+))"
    r"(?:[eE](?P<exponent>[+-]?\d+))?$"
)


def exact_scaled_integer(value: ExactDecimal, decimals: int) -> int:
    """Scale a decimal exactly, failing instead of rounding or truncating."""
    if isinstance(value, float) or not isinstance(value, (str, Decimal)):
        raise ScalingError("decimal values must be strings or Decimal instances")
    if not isinstance(decimals, int) or decimals < 0:
        raise ScalingError(f"invalid decimal scale: {decimals}")
    source = str(value).strip()
    if not source:
        raise ScalingError("invalid decimal: empty input")
    if source.startswith("-"):
        raise ScalingError(f"invalid decimal '{source}': negative values are not supported")
    match = _DECIMAL_RE.fullmatch(source)
    if match is None:
        raise ScalingError(f"invalid decimal '{source}': expected base-10 decimal syntax")
    whole = match.group("whole") or ""
    fraction = match.group("fraction")
    if fraction is None:
        fraction = match.group("leading_fraction") or ""
    exponent = int(match.group("exponent") or 0)
    if abs(exponent) > 10_000:
        raise ScalingError(f"invalid decimal '{source}': unsupported exponent")
    coefficient = (whole + fraction).lstrip("0")
    if not coefficient:
        return 0
    shift = decimals + exponent - len(fraction)
    if shift >= 0:
        if len(coefficient) + shift > 10_000:
            raise ScalingError(f"invalid decimal '{source}': scaled value is too large")
        digits = coefficient + "0" * shift
    else:
        remove = -shift
        if remove > len(coefficient) or any(char != "0" for char in coefficient[-remove:]):
            raise ScalingError(
                f"invalid decimal '{source}': cannot be represented exactly at this scale"
            )
        digits = coefficient[:-remove] or "0"
    return int(digits)


def _significant_digits(value: int) -> int:
    while value and value % 10 == 0:
        value //= 10
    return len(str(value))


def _validate_price_raw(price_raw: int, rules: OrderbookRules) -> int:
    if price_raw <= 0 or price_raw > I64_MAX:
        raise ScalingError.from_code("PRICE_OUT_OF_RANGE")
    human_scale = 10**rules.price_decimals
    integer_price = price_raw % human_scale == 0
    trading = rules.trading_rules
    if not integer_price:
        if trading.price_quantum_raw <= 0 or price_raw % trading.price_quantum_raw != 0:
            raise ScalingError.from_code("INVALID_PRICE_DECIMALS")
        if _significant_digits(price_raw) > trading.max_price_significant_figures:
            raise ScalingError.from_code("INVALID_PRICE_SIGNIFICANT_FIGURES")
    return price_raw


def _validate_base_atoms(base_atoms: int, rules: OrderbookRules) -> int:
    if base_atoms <= 0:
        raise ScalingError(f"size must be positive, got {base_atoms}")
    quantum = rules.trading_rules.base_size_quantum_raw
    if quantum <= 0 or base_atoms % quantum != 0:
        raise ScalingError.from_code("INVALID_SIZE_DECIMALS")
    validate_signed_field(base_atoms, "base amount", zero_allowed=False)
    return base_atoms


def scale_price_size(
    price: ExactDecimal,
    size: ExactDecimal,
    side: int,
    rules: OrderbookRules,
) -> ScaledAmounts:
    """Construct the exact signed amounts for a human price and base size."""
    price_raw = exact_scaled_integer(price, rules.price_decimals)
    if price_raw <= 0:
        raise ScalingError(f"price must be positive, got {price}")
    base_atoms = exact_scaled_integer(size, rules.base_decimals)
    if base_atoms <= 0:
        raise ScalingError(f"size must be positive, got {size}")
    _validate_price_raw(price_raw, rules)
    _validate_base_atoms(base_atoms, rules)
    quote_numerator = price_raw * base_atoms
    if quote_numerator % PRICE_SCALE:
        raise ScalingError.from_code("PRICE_NOT_EXACTLY_REPRESENTABLE")
    quote_atoms = quote_numerator // PRICE_SCALE
    validate_signed_field(quote_atoms, "quote amount", zero_allowed=False)
    if int(side) == 0:
        amount_in, amount_out = quote_atoms, base_atoms
    elif int(side) == 1:
        amount_in, amount_out = base_atoms, quote_atoms
    else:
        raise ScalingError("side must be BID or ASK")
    return ScaledAmounts(
        amount_in=amount_in,
        amount_out=amount_out,
        price_raw=price_raw,
        base_atoms=base_atoms,
        quote_atoms=quote_atoms,
    )


def validate_raw_amounts(
    amount_in: int,
    amount_out: int,
    side: int,
    rules: OrderbookRules,
) -> ScaledAmounts:
    """Preflight caller-provided signed amounts against the engine rules."""
    validate_signed_field(amount_in, "amount_in", zero_allowed=False)
    validate_signed_field(amount_out, "amount_out", zero_allowed=False)
    if int(side) == 0:
        base_atoms, quote_atoms = amount_out, amount_in
    elif int(side) == 1:
        base_atoms, quote_atoms = amount_in, amount_out
    else:
        raise ScalingError("side must be BID or ASK")
    _validate_base_atoms(base_atoms, rules)
    numerator = quote_atoms * PRICE_SCALE
    if numerator % base_atoms:
        raise ScalingError.from_code("PRICE_NOT_EXACTLY_REPRESENTABLE")
    price_raw = numerator // base_atoms
    _validate_price_raw(price_raw, rules)
    return ScaledAmounts(
        amount_in=amount_in,
        amount_out=amount_out,
        price_raw=price_raw,
        base_atoms=base_atoms,
        quote_atoms=quote_atoms,
    )


def validate_signed_field(value: int, field: str, *, zero_allowed: bool = True) -> None:
    if not isinstance(value, int) or value < 0 or value > I64_MAX or (not zero_allowed and value == 0):
        raise ScalingError.from_code("ORDER_FIELD_OUT_OF_RANGE", field)


def validate_signed_fields(amount_in: int, amount_out: int, salt: int, nonce: int) -> None:
    validate_signed_field(amount_in, "amount_in", zero_allowed=False)
    validate_signed_field(amount_out, "amount_out", zero_allowed=False)
    validate_signed_field(salt, "salt")
    if not isinstance(nonce, int) or nonce < 0 or nonce > U32_MAX:
        raise ScalingError.from_code("ORDER_FIELD_OUT_OF_RANGE", "nonce")


def validate_trigger_price(value: ExactDecimal, price_decimals: int) -> int:
    try:
        raw = exact_scaled_integer(value, price_decimals)
    except ScalingError as exc:
        raise ScalingError.from_code("TRIGGER_PRICE_OUT_OF_RANGE") from exc
    if raw <= 0 or raw > I64_MAX:
        raise ScalingError.from_code("TRIGGER_PRICE_OUT_OF_RANGE")
    return raw
