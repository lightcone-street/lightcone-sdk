"""Shared formatter precision constants."""

from collections.abc import Callable

DISPLAY_DECIMAL_TIERS: tuple[tuple[str, int], ...] = (
    ("10000", 0),
    ("1000", 1),
    ("100", 2),
    ("10", 3),
    ("0.1", 4),
)
SMALL_VALUE_DECIMALS = 5


def display_decimals_by(matches_tier: Callable[[str], bool]) -> int:
    for threshold, decimals in DISPLAY_DECIMAL_TIERS:
        if matches_tier(threshold):
            return decimals

    return SMALL_VALUE_DECIMALS
