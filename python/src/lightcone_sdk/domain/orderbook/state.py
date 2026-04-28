"""Orderbook state for WebSocket updates."""

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Optional, Union


@dataclass(frozen=True)
class OrderbookIgnoreReason:
    kind: str
    current: Optional[int] = None
    got: Optional[int] = None

    @staticmethod
    def invalid_delta_sequence(got: int) -> "OrderbookIgnoreReason":
        return OrderbookIgnoreReason(kind="invalid_delta_sequence", got=got)

    @staticmethod
    def stale_delta(current: int, got: int) -> "OrderbookIgnoreReason":
        return OrderbookIgnoreReason(kind="stale_delta", current=current, got=got)

    @staticmethod
    def already_awaiting_snapshot(got: int) -> "OrderbookIgnoreReason":
        return OrderbookIgnoreReason(kind="already_awaiting_snapshot", got=got)


@dataclass(frozen=True)
class OrderbookRefreshReason:
    kind: str
    expected: Optional[int] = None
    got: Optional[int] = None

    @staticmethod
    def missing_snapshot(got: int) -> "OrderbookRefreshReason":
        return OrderbookRefreshReason(kind="missing_snapshot", got=got)

    @staticmethod
    def sequence_gap(expected: int, got: int) -> "OrderbookRefreshReason":
        return OrderbookRefreshReason(kind="sequence_gap", expected=expected, got=got)

    @staticmethod
    def server_resync(got: int) -> "OrderbookRefreshReason":
        return OrderbookRefreshReason(kind="server_resync", got=got)


@dataclass(frozen=True)
class OrderbookApplyResult:
    kind: str
    reason: Optional[Union[OrderbookIgnoreReason, OrderbookRefreshReason]] = None

    @staticmethod
    def applied() -> "OrderbookApplyResult":
        return OrderbookApplyResult(kind="applied")

    @staticmethod
    def ignored(reason: OrderbookIgnoreReason) -> "OrderbookApplyResult":
        return OrderbookApplyResult(kind="ignored", reason=reason)

    @staticmethod
    def refresh_required(reason: OrderbookRefreshReason) -> "OrderbookApplyResult":
        return OrderbookApplyResult(kind="refresh_required", reason=reason)


@dataclass
class OrderbookState:
    """Local orderbook state maintained from WebSocket updates."""

    orderbook_id: str
    bids: dict[str, str] = field(default_factory=dict)
    asks: dict[str, str] = field(default_factory=dict)
    sequence: int = 0
    _has_snapshot: bool = field(default=False, init=False, repr=False)
    _awaiting_snapshot: bool = field(default=False, init=False, repr=False)

    def apply(self, update) -> OrderbookApplyResult:
        """Apply a book update (snapshot or delta).

        Accepts either a raw dict or a WsOrderBook dataclass.
        """
        if hasattr(update, "is_snapshot"):
            return self._apply_typed(update)
        return self._apply_dict(update)

    def _apply_typed(self, update) -> OrderbookApplyResult:
        """Apply a WsOrderBook dataclass.

        Server resync messages take precedence and return refresh_required.
        Otherwise, snapshots are applied and deltas with a seq at or below the
        current value are ignored to prevent stale updates. Deltas that skip
        one or more expected sequence values are rejected so callers can refresh
        from a fresh snapshot instead of mutating a corrupted book.
        """
        if getattr(update, "resync", False):
            self._awaiting_snapshot = True
            return OrderbookApplyResult.refresh_required(
                OrderbookRefreshReason.server_resync(update.seq)
            )

        if update.is_snapshot:
            self.bids.clear()
            self.asks.clear()
            self._has_snapshot = True
            self._awaiting_snapshot = False
        else:
            if self._awaiting_snapshot:
                return OrderbookApplyResult.ignored(
                    OrderbookIgnoreReason.already_awaiting_snapshot(update.seq)
                )
            # The backend sends snapshots with seq=0 and starts delta seq at 1.
            # A delta with seq=0 means it has no valid sequence, so drop it.
            if update.seq <= 0:
                return OrderbookApplyResult.ignored(
                    OrderbookIgnoreReason.invalid_delta_sequence(update.seq)
                )
            if not self._has_snapshot:
                self._awaiting_snapshot = True
                return OrderbookApplyResult.refresh_required(
                    OrderbookRefreshReason.missing_snapshot(update.seq)
                )
            if update.seq <= self.sequence:
                return OrderbookApplyResult.ignored(
                    OrderbookIgnoreReason.stale_delta(self.sequence, update.seq)
                )
            expected = self.sequence + 1
            if update.seq != expected:
                self._awaiting_snapshot = True
                return OrderbookApplyResult.refresh_required(
                    OrderbookRefreshReason.sequence_gap(expected, update.seq)
                )

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

        Server resync messages take precedence and return refresh_required.
        Otherwise, snapshots are applied and deltas with a seq at or below the
        current value are ignored to prevent stale updates. Deltas that skip
        one or more expected sequence values are rejected so callers can refresh
        from a fresh snapshot instead of mutating a corrupted book.
        """
        seq = update.get("seq", 0)
        if update.get("resync", False):
            self._awaiting_snapshot = True
            return OrderbookApplyResult.refresh_required(
                OrderbookRefreshReason.server_resync(seq)
            )

        is_snapshot = update.get("is_snapshot", False)
        if is_snapshot:
            self.bids.clear()
            self.asks.clear()
            self._has_snapshot = True
            self._awaiting_snapshot = False
        else:
            seq = update.get("seq", 0)
            if self._awaiting_snapshot:
                return OrderbookApplyResult.ignored(
                    OrderbookIgnoreReason.already_awaiting_snapshot(seq)
                )
            # The backend sends snapshots with seq=0 and starts delta seq at 1.
            # A delta with seq=0 means it has no valid sequence, so drop it.
            if seq <= 0:
                return OrderbookApplyResult.ignored(
                    OrderbookIgnoreReason.invalid_delta_sequence(seq)
                )
            if not self._has_snapshot:
                self._awaiting_snapshot = True
                return OrderbookApplyResult.refresh_required(
                    OrderbookRefreshReason.missing_snapshot(seq)
                )
            if seq <= self.sequence:
                return OrderbookApplyResult.ignored(
                    OrderbookIgnoreReason.stale_delta(self.sequence, seq)
                )
            expected = self.sequence + 1
            if seq != expected:
                self._awaiting_snapshot = True
                return OrderbookApplyResult.refresh_required(
                    OrderbookRefreshReason.sequence_gap(expected, seq)
                )

        for bid in update.get("bids", []):
            price = str(bid.get("price", bid[0] if isinstance(bid, list) else "0"))
            size = str(
                bid.get(
                    "size", bid[1] if isinstance(bid, list) and len(bid) > 1 else "0"
                )
            )
            if size == "0":
                self.bids.pop(price, None)
            else:
                self.bids[price] = size

        for ask in update.get("asks", []):
            price = str(ask.get("price", ask[0] if isinstance(ask, list) else "0"))
            size = str(
                ask.get(
                    "size", ask[1] if isinstance(ask, list) and len(ask) > 1 else "0"
                )
            )
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
        self._has_snapshot = False
        self._awaiting_snapshot = False
