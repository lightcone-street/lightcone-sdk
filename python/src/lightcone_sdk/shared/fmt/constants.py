"""Shared formatter precision constants."""

from collections.abc import Callable
from decimal import Decimal

DISPLAY_DECIMAL_TIERS: tuple[tuple[Decimal, int], ...] = (
    (Decimal("10000"), 0),
    (Decimal("1000"), 1),
    (Decimal("100"), 2),
    (Decimal("10"), 3),
    (Decimal("0.1"), 4),
)
SMALL_VALUE_DECIMALS = 5


def display_decimals_by(matches_tier: Callable[[Decimal], bool]) -> int:
    for threshold, decimals in DISPLAY_DECIMAL_TIERS:
        if matches_tier(threshold):
            return decimals

    return SMALL_VALUE_DECIMALS
