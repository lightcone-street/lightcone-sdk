"""Shared type definitions used by API and WebSocket modules."""

from enum import IntEnum
from typing import ClassVar


class Resolution(IntEnum):
    """Price history candle resolution."""

    ONE_MINUTE = 0
    FIVE_MINUTES = 1
    FIFTEEN_MINUTES = 2
    ONE_HOUR = 3
    FOUR_HOURS = 4
    ONE_DAY = 5

    _STR_MAP: ClassVar[dict[int, str]] = {
        0: "1m",
        1: "5m",
        2: "15m",
        3: "1h",
        4: "4h",
        5: "1d",
    }
    _FROM_STR_MAP: ClassVar[dict[str, int]] = {v: k for k, v in _STR_MAP.items()}

    def as_str(self) -> str:
        """Get the string representation for API calls."""
        return self._STR_MAP[self.value]

    @classmethod
    def from_str(cls, s: str) -> "Resolution":
        """Parse a resolution string."""
        if s not in cls._FROM_STR_MAP:
            raise ValueError(f"Invalid resolution: {s}")
        return cls(cls._FROM_STR_MAP[s])

    def __str__(self) -> str:
        return self.as_str()
