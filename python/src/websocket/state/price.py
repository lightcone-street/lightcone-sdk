"""Price history state management."""

from dataclasses import dataclass
from typing import Optional

from ..types import Candle, PriceHistoryData
from ...shared.types import Resolution


@dataclass
class PriceHistoryKey:
    """Key for price history subscriptions."""

    orderbook_id: str
    resolution: str

    def __hash__(self):
        return hash((self.orderbook_id, self.resolution))

    def __eq__(self, other):
        if not isinstance(other, PriceHistoryKey):
            return False
        return (
            self.orderbook_id == other.orderbook_id
            and self.resolution == other.resolution
        )


class PriceHistory:
    """Price history state for a single orderbook/resolution pair."""

    def __init__(self, orderbook_id: str, resolution: str, include_ohlcv: bool = False):
        self.orderbook_id = orderbook_id
        self.resolution = resolution
        self.include_ohlcv = include_ohlcv
        self._candles: list[Candle] = []  # Sorted by timestamp (newest first)
        self._candle_index: dict[int, int] = {}  # timestamp -> index
        self._last_timestamp: Optional[int] = None
        self._server_time: Optional[int] = None
        self._has_snapshot: bool = False

    def apply_snapshot(self, data: PriceHistoryData) -> None:
        """Apply a snapshot (historical candles)."""
        self._candles.clear()
        self._candle_index.clear()

        # Apply candles (they come newest-first from server)
        for candle in data.prices:
            idx = len(self._candles)
            self._candle_index[candle.t] = idx
            self._candles.append(candle)

        self._last_timestamp = data.last_timestamp
        self._server_time = data.server_time
        self._has_snapshot = True

        if data.include_ohlcv is not None:
            self.include_ohlcv = data.include_ohlcv

    def apply_update(self, data: PriceHistoryData) -> None:
        """Apply an update (new or updated candle)."""
        candle = data.to_candle()
        if candle:
            self._update_or_append_candle(candle)

    def _update_or_append_candle(self, candle: Candle) -> None:
        """Update an existing candle or append a new one."""
        if candle.t in self._candle_index:
            # Update existing candle
            idx = self._candle_index[candle.t]
            self._candles[idx] = candle
        else:
            # New candle - insert at the correct position (newest first)
            insert_pos = 0
            for i, c in enumerate(self._candles):
                if c.t < candle.t:
                    insert_pos = i
                    break
            else:
                insert_pos = len(self._candles)

            # Update indices for candles that will shift
            new_index = {}
            for ts, idx in self._candle_index.items():
                if idx >= insert_pos:
                    new_index[ts] = idx + 1
                else:
                    new_index[ts] = idx
            new_index[candle.t] = insert_pos
            self._candle_index = new_index

            self._candles.insert(insert_pos, candle)

            # Trim to max 1000 candles
            while len(self._candles) > 1000:
                removed = self._candles.pop()
                self._candle_index.pop(removed.t, None)

        # Update last timestamp
        if self._candles:
            self._last_timestamp = self._candles[0].t

    def apply_heartbeat(self, data: PriceHistoryData) -> None:
        """Handle heartbeat (update server time)."""
        self._server_time = data.server_time

    def apply_event(self, data: PriceHistoryData) -> None:
        """Apply any price history event."""
        if data.event_type == "snapshot":
            self.apply_snapshot(data)
        elif data.event_type == "update":
            self.apply_update(data)
        elif data.event_type == "heartbeat":
            self.apply_heartbeat(data)

    def candles(self) -> list[Candle]:
        """Get all candles (newest first)."""
        return self._candles.copy()

    def recent_candles(self, n: int) -> list[Candle]:
        """Get the N most recent candles."""
        return self._candles[:n]

    def get_candle(self, timestamp: int) -> Optional[Candle]:
        """Get a candle by timestamp."""
        idx = self._candle_index.get(timestamp)
        if idx is not None:
            return self._candles[idx]
        return None

    def latest_candle(self) -> Optional[Candle]:
        """Get the most recent candle."""
        return self._candles[0] if self._candles else None

    def oldest_candle(self) -> Optional[Candle]:
        """Get the oldest candle."""
        return self._candles[-1] if self._candles else None

    def current_midpoint(self) -> Optional[str]:
        """Get current midpoint price (from most recent candle)."""
        if self._candles:
            return self._candles[0].m
        return None

    def current_best_bid(self) -> Optional[str]:
        """Get current best bid (from most recent candle)."""
        if self._candles:
            return self._candles[0].bb
        return None

    def current_best_ask(self) -> Optional[str]:
        """Get current best ask (from most recent candle)."""
        if self._candles:
            return self._candles[0].ba
        return None

    def candle_count(self) -> int:
        """Number of candles."""
        return len(self._candles)

    def has_snapshot(self) -> bool:
        """Whether the price history has received its initial snapshot."""
        return self._has_snapshot

    def last_timestamp(self) -> Optional[int]:
        """Last candle timestamp."""
        return self._last_timestamp

    def server_time(self) -> Optional[int]:
        """Server time from last message."""
        return self._server_time

    def resolution_enum(self) -> Optional[Resolution]:
        """Get resolution as enum."""
        try:
            return Resolution.from_str(self.resolution)
        except ValueError:
            return None

    def clear(self) -> None:
        """Clear the price history (for disconnect/resync)."""
        self._candles.clear()
        self._candle_index.clear()
        self._last_timestamp = None
        self._server_time = None
        self._has_snapshot = False
