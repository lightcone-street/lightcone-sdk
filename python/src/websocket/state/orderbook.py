"""Local orderbook state management."""

from typing import Optional
from collections import OrderedDict

from ..types import BookUpdateData, PriceLevel
from ..error import SequenceGapError


def is_zero_size(size: str) -> bool:
    """Check if a size string represents zero."""
    if size in ("0", "0.0", "0.000000"):
        return True
    try:
        return float(size) == 0.0
    except (ValueError, TypeError):
        return False


class LocalOrderbook:
    """Local orderbook state.

    Maintains a local copy of the orderbook state, applying deltas from
    WebSocket updates.
    """

    def __init__(self, orderbook_id: str):
        self.orderbook_id = orderbook_id
        # Use dict for O(1) lookup, maintain sorted order via BTreeMap-like behavior
        self._bids: dict[str, str] = {}  # price -> size
        self._asks: dict[str, str] = {}  # price -> size
        self._expected_seq: int = 0
        self._has_snapshot: bool = False
        self._last_timestamp: Optional[str] = None

    def apply_snapshot(self, update: BookUpdateData) -> None:
        """Apply a snapshot (full orderbook state)."""
        self._bids.clear()
        self._asks.clear()

        for level in update.bids:
            if not is_zero_size(level.size):
                self._bids[level.price] = level.size

        for level in update.asks:
            if not is_zero_size(level.size):
                self._asks[level.price] = level.size

        self._expected_seq = update.seq + 1
        self._has_snapshot = True
        self._last_timestamp = update.timestamp

    def apply_delta(self, update: BookUpdateData) -> None:
        """Apply a delta update.

        Raises:
            SequenceGapError: If a sequence gap is detected
        """
        if update.seq != self._expected_seq:
            raise SequenceGapError(self._expected_seq, update.seq)

        for level in update.bids:
            if is_zero_size(level.size):
                self._bids.pop(level.price, None)
            else:
                self._bids[level.price] = level.size

        for level in update.asks:
            if is_zero_size(level.size):
                self._asks.pop(level.price, None)
            else:
                self._asks[level.price] = level.size

        self._expected_seq = update.seq + 1
        self._last_timestamp = update.timestamp

    def apply_update(self, update: BookUpdateData) -> None:
        """Apply an update (snapshot or delta).

        Raises:
            SequenceGapError: If a sequence gap is detected in delta
        """
        if update.is_snapshot:
            self.apply_snapshot(update)
        else:
            self.apply_delta(update)

    def get_bids(self) -> list[PriceLevel]:
        """Get all bid levels sorted by price (descending)."""
        # Sort by price descending (highest first)
        sorted_prices = sorted(self._bids.keys(), key=float, reverse=True)
        return [
            PriceLevel(side="bid", price=price, size=self._bids[price])
            for price in sorted_prices
        ]

    def get_asks(self) -> list[PriceLevel]:
        """Get all ask levels sorted by price (ascending)."""
        # Sort by price ascending (lowest first)
        sorted_prices = sorted(self._asks.keys(), key=float)
        return [
            PriceLevel(side="ask", price=price, size=self._asks[price])
            for price in sorted_prices
        ]

    def get_top_bids(self, n: int) -> list[PriceLevel]:
        """Get top N bid levels."""
        return self.get_bids()[:n]

    def get_top_asks(self, n: int) -> list[PriceLevel]:
        """Get top N ask levels."""
        return self.get_asks()[:n]

    def best_bid(self) -> Optional[tuple[str, str]]:
        """Get the best bid (highest bid price) as (price, size)."""
        if not self._bids:
            return None
        price = max(self._bids.keys(), key=float)
        return (price, self._bids[price])

    def best_ask(self) -> Optional[tuple[str, str]]:
        """Get the best ask (lowest ask price) as (price, size)."""
        if not self._asks:
            return None
        price = min(self._asks.keys(), key=float)
        return (price, self._asks[price])

    def spread(self) -> Optional[str]:
        """Get the spread as a string (best_ask - best_bid)."""
        bid = self.best_bid()
        ask = self.best_ask()
        if bid is None or ask is None:
            return None
        try:
            bid_f = float(bid[0])
            ask_f = float(ask[0])
            if ask_f > bid_f:
                return f"{ask_f - bid_f:.6f}"
            return "0.000000"
        except (ValueError, TypeError):
            return None

    def midpoint(self) -> Optional[str]:
        """Get the midpoint price as a string."""
        bid = self.best_bid()
        ask = self.best_ask()
        if bid is None or ask is None:
            return None
        try:
            bid_f = float(bid[0])
            ask_f = float(ask[0])
            return f"{(bid_f + ask_f) / 2:.6f}"
        except (ValueError, TypeError):
            return None

    def bid_size_at(self, price: str) -> Optional[str]:
        """Get size at a specific bid price."""
        return self._bids.get(price)

    def ask_size_at(self, price: str) -> Optional[str]:
        """Get size at a specific ask price."""
        return self._asks.get(price)

    def total_bid_depth(self) -> float:
        """Get total bid depth (sum of all bid sizes)."""
        total = 0.0
        for size in self._bids.values():
            try:
                total += float(size)
            except (ValueError, TypeError):
                pass
        return total

    def total_ask_depth(self) -> float:
        """Get total ask depth (sum of all ask sizes)."""
        total = 0.0
        for size in self._asks.values():
            try:
                total += float(size)
            except (ValueError, TypeError):
                pass
        return total

    def bid_count(self) -> int:
        """Number of bid levels."""
        return len(self._bids)

    def ask_count(self) -> int:
        """Number of ask levels."""
        return len(self._asks)

    def has_snapshot(self) -> bool:
        """Whether the orderbook has received its initial snapshot."""
        return self._has_snapshot

    def expected_sequence(self) -> int:
        """Current expected sequence number."""
        return self._expected_seq

    def last_timestamp(self) -> Optional[str]:
        """Last update timestamp."""
        return self._last_timestamp

    def clear(self) -> None:
        """Clear the orderbook state (for resync)."""
        self._bids.clear()
        self._asks.clear()
        self._expected_seq = 0
        self._has_snapshot = False
        self._last_timestamp = None
