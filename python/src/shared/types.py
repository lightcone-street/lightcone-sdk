"""Shared type definitions used by API and WebSocket modules."""

from enum import IntEnum


class Resolution(IntEnum):
    """Price history candle resolution."""

    ONE_MINUTE = 0
    FIVE_MINUTES = 1
    FIFTEEN_MINUTES = 2
    ONE_HOUR = 3
    FOUR_HOURS = 4
    ONE_DAY = 5

    def as_str(self) -> str:
        """Get the string representation for API calls."""
        mapping = {
            Resolution.ONE_MINUTE: "1m",
            Resolution.FIVE_MINUTES: "5m",
            Resolution.FIFTEEN_MINUTES: "15m",
            Resolution.ONE_HOUR: "1h",
            Resolution.FOUR_HOURS: "4h",
            Resolution.ONE_DAY: "1d",
        }
        return mapping[self]

    @classmethod
    def from_str(cls, s: str) -> "Resolution":
        """Parse a resolution string."""
        mapping = {
            "1m": cls.ONE_MINUTE,
            "5m": cls.FIVE_MINUTES,
            "15m": cls.FIFTEEN_MINUTES,
            "1h": cls.ONE_HOUR,
            "4h": cls.FOUR_HOURS,
            "1d": cls.ONE_DAY,
        }
        if s not in mapping:
            raise ValueError(f"Invalid resolution: {s}")
        return mapping[s]

    def __str__(self) -> str:
        return self.as_str()
