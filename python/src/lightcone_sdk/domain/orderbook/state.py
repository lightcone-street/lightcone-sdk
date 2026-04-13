"""Orderbook state for WebSocket updates."""

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Optional, Union


@dataclass(frozen=True)
class OrderbookApplyResult:
    kind: str
    expected: Optional[int] = None
    got: Optional[int] = None

    @staticmethod
    def applied() -> "OrderbookApplyResult":
        return OrderbookApplyResult(kind="applied")

    @staticmethod
    def ignored_stale() -> "OrderbookApplyResult":
        return OrderbookApplyResult(kind="ignored_stale")

    @staticmethod
    def gap_detected(expected: int, got: int) -> "OrderbookApplyResult":
        return OrderbookApplyResult(kind="gap_detected", expected=expected, got=got)


@dataclass
class OrderbookSnapshot:
    """Local orderbook state maintained from WebSocket updates."""
    orderbook_id: str
    bids: dict[str, str] = field(default_factory=dict)
    asks: dict[str, str] = field(default_factory=dict)
    sequence: int = 0

    def apply(self, update) -> OrderbookApplyResult:
        """Apply a book update (snapshot or delta).

        Accepts either a raw dict or a WsOrderBook dataclass.
        """
        if hasattr(update, "is_snapshot"):
            return self._apply_typed(update)
        return self._apply_dict(update)

    def _apply_typed(self, update) -> OrderbookApplyResult:
        """Apply a WsOrderBook dataclass.

        Snapshots are always applied. Deltas with a seq at or below the
        current value are silently dropped to prevent stale updates. Deltas
        that skip one or more expected sequence values are rejected so callers
        can refresh from a fresh snapshot instead of mutating a corrupted book.
        """
        if update.is_snapshot:
            self.bids.clear()
            self.asks.clear()
        else:
            # The backend sends snapshots with seq=0 and starts delta seq at 1.
            # A delta with seq=0 means it has no valid sequence, so drop it.
            if update.seq <= 0:
                return OrderbookApplyResult.ignored_stale()
            if self.sequence == 0:
                return OrderbookApplyResult.gap_detected(0, update.seq)
            if update.seq <= self.sequence:
                return OrderbookApplyResult.ignored_stale()
            if update.seq != self.sequence + 1:
                return OrderbookApplyResult.gap_detected(self.sequence + 1, update.seq)

        for bid in update.bids:
            if bid.size == "0":
                self.bids.pop(bid.price, None)
            else:
                self.bids[bid.price] = bid.size

        for ask in update.asks:
            if ask.size == "0":
                self.asks.pop(ask.price, None)
            else:
                self.asks[ask.price] = ask.size

        self.sequence = update.seq
        return OrderbookApplyResult.applied()

    def _apply_dict(self, update: dict) -> OrderbookApplyResult:
        """Apply a raw dict update.

        Snapshots are always applied. Deltas with a seq at or below the
        current value are silently dropped to prevent stale updates. Deltas
        that skip one or more expected sequence values are rejected so callers
        can refresh from a fresh snapshot instead of mutating a corrupted book.
        """
        is_snapshot = update.get("is_snapshot", False)
        if is_snapshot:
            self.bids.clear()
            self.asks.clear()
        else:
            seq = update.get("seq", 0)
            # The backend sends snapshots with seq=0 and starts delta seq at 1.
            # A delta with seq=0 means it has no valid sequence, so drop it.
            if seq <= 0:
                return OrderbookApplyResult.ignored_stale()
            if self.sequence == 0:
                return OrderbookApplyResult.gap_detected(0, seq)
            if seq <= self.sequence:
                return OrderbookApplyResult.ignored_stale()
            if seq != self.sequence + 1:
                return OrderbookApplyResult.gap_detected(self.sequence + 1, seq)

        for bid in update.get("bids", []):
            price = str(bid.get("price", bid[0] if isinstance(bid, list) else "0"))
            size = str(bid.get("size", bid[1] if isinstance(bid, list) and len(bid) > 1 else "0"))
            if size == "0":
                self.bids.pop(price, None)
            else:
                self.bids[price] = size

        for ask in update.get("asks", []):
            price = str(ask.get("price", ask[0] if isinstance(ask, list) else "0"))
            size = str(ask.get("size", ask[1] if isinstance(ask, list) and len(ask) > 1 else "0"))
            if size == "0":
                self.asks.pop(price, None)
            else:
                self.asks[price] = size

        seq = update.get("seq")
        if seq is not None:
            self.sequence = seq
        return OrderbookApplyResult.applied()

    def best_bid(self) -> Optional[str]:
        if not self.bids:
            return None
        return max(self.bids.keys(), key=lambda p: float(p))

    def best_ask(self) -> Optional[str]:
        if not self.asks:
            return None
        return min(self.asks.keys(), key=lambda p: float(p))

    def mid_price(self) -> Optional[str]:
        bb = self.best_bid()
        ba = self.best_ask()
        if bb is None or ba is None:
            return None
        return str((Decimal(bb) + Decimal(ba)) / 2)

    def spread(self) -> Optional[str]:
        bb = self.best_bid()
        ba = self.best_ask()
        if bb is None or ba is None:
            return None
        return str(Decimal(ba) - Decimal(bb))

    def is_empty(self) -> bool:
        return not self.bids and not self.asks

    def clear(self) -> None:
        self.bids.clear()
        self.asks.clear()
        self.sequence = 0
