"""Orderbook state for WebSocket updates.

The ``book_update`` stream is snapshot-only: every data frame carries the
full top-20 levels per side and replaces the previous book wholesale.
Equal and older revisions are discarded within one subscription generation.
Consumers holding multiple aggregation views of one orderbook on the same
connection key their :class:`OrderbookState` instances by
``(orderbook_id, aggregation)`` using ``WsOrderBook.aggregation()``.
"""

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Optional

from .aggregation import BookAggregation, FULL_PRECISION


@dataclass(frozen=True)
class OrderbookRefreshReason:
    kind: str

    @staticmethod
    def server_resync() -> "OrderbookRefreshReason":
        """The backend explicitly requested a resync: unsubscribe and
        re-subscribe with the same parameters (including aggregation) to
        receive a fresh snapshot."""
        return OrderbookRefreshReason(kind="server_resync")


@dataclass(frozen=True)
class OrderbookApplyResult:
    kind: str
    reason: Optional[OrderbookRefreshReason] = None

    @staticmethod
    def applied() -> "OrderbookApplyResult":
        return OrderbookApplyResult(kind="applied")

    @staticmethod
    def refresh_required(reason: OrderbookRefreshReason) -> "OrderbookApplyResult":
        return OrderbookApplyResult(kind="refresh_required", reason=reason)

    @staticmethod
    def discarded_stale() -> "OrderbookApplyResult":
        return OrderbookApplyResult(kind="discarded_stale")

    @staticmethod
    def subscription_mismatch() -> "OrderbookApplyResult":
        return OrderbookApplyResult(kind="subscription_mismatch")


@dataclass
class OrderbookState:
    """Local orderbook state replaced wholesale by snapshot frames.

    ``sequence`` is the last accepted engine revision. Forward gaps are normal.
    """

    orderbook_id: str
    bids: dict[str, str] = field(default_factory=dict)
    asks: dict[str, str] = field(default_factory=dict)
    sequence: int = 0
    aggregation: BookAggregation = FULL_PRECISION
    bids_truncated: bool = False
    asks_truncated: bool = False
    _last_sequence: Optional[int] = field(default=None, init=False, repr=False)

    def __post_init__(self) -> None:
        self.aggregation = self.aggregation.normalized()

    def apply(self, update) -> OrderbookApplyResult:
        """Apply a full snapshot when its revision is newer in this generation.

        Accepts either a raw dict or a WsOrderBook dataclass. ``resync``
        frames take precedence and leave the book untouched — the caller must
        re-subscribe with the same parameters. Every other data frame is a
        full snapshot by contract and replaces the book wholesale. Revision
        gaps are expected and do not request resync.
        """
        if hasattr(update, "is_snapshot"):
            return self._apply_typed(update)
        return self._apply_dict(update)

    def _apply_typed(self, update) -> OrderbookApplyResult:
        if update.seq < 0:
            raise ValueError("book_update seq must be non-negative")
        if (
            update.orderbook_id != self.orderbook_id
            or update.aggregation() != self.aggregation
        ):
            return OrderbookApplyResult.subscription_mismatch()
        if getattr(update, "resync", False):
            return OrderbookApplyResult.refresh_required(
                OrderbookRefreshReason.server_resync()
            )
        if self._last_sequence is not None and update.seq <= self._last_sequence:
            return OrderbookApplyResult.discarded_stale()

        self.bids.clear()
        self.asks.clear()
        for bid in update.bids:
            if bid.size != "0":
                self.bids[bid.price] = bid.size
        for ask in update.asks:
            if ask.size != "0":
                self.asks[ask.price] = ask.size
        self.sequence = update.seq
        self._last_sequence = update.seq
        self.bids_truncated = bool(getattr(update, "bids_truncated", False))
        self.asks_truncated = bool(getattr(update, "asks_truncated", False))
        return OrderbookApplyResult.applied()

    def _apply_dict(self, update: dict) -> OrderbookApplyResult:
        update_id = update.get("orderbook_id") or update.get("id")
        update_aggregation = BookAggregation.from_frame(
            update.get("n_sig_figs"), update.get("mantissa")
        )
        if update_id != self.orderbook_id or update_aggregation != self.aggregation:
            return OrderbookApplyResult.subscription_mismatch()
        if update.get("resync", False):
            return OrderbookApplyResult.refresh_required(
                OrderbookRefreshReason.server_resync()
            )
        if "seq" not in update:
            raise ValueError("book_update seq is required")
        sequence = update["seq"]
        if (
            not isinstance(sequence, int)
            or isinstance(sequence, bool)
            or sequence < 0
        ):
            raise ValueError("book_update seq must be a non-negative integer")
        if self._last_sequence is not None and sequence <= self._last_sequence:
            return OrderbookApplyResult.discarded_stale()

        self.bids.clear()
        self.asks.clear()
        for bid in update.get("bids", []):
            price = str(bid.get("price", bid[0] if isinstance(bid, list) else "0"))
            size = str(
                bid.get(
                    "size", bid[1] if isinstance(bid, list) and len(bid) > 1 else "0"
                )
            )
            if size != "0":
                self.bids[price] = size

        for ask in update.get("asks", []):
            price = str(ask.get("price", ask[0] if isinstance(ask, list) else "0"))
            size = str(
                ask.get(
                    "size", ask[1] if isinstance(ask, list) and len(ask) > 1 else "0"
                )
            )
            if size != "0":
                self.asks[price] = size

        self.sequence = sequence
        self._last_sequence = sequence
        self.bids_truncated = bool(update.get("bids_truncated", False))
        self.asks_truncated = bool(update.get("asks_truncated", False))
        return OrderbookApplyResult.applied()

    def best_bid(self) -> Optional[str]:
        if not self.bids:
            return None
        return max(self.bids.keys(), key=Decimal)

    def best_ask(self) -> Optional[str]:
        if not self.asks:
            return None
        return min(self.asks.keys(), key=Decimal)

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

    def begin_generation(self) -> None:
        """Reset the revision gate for reconnect/resubscribe, preserving levels."""
        self._last_sequence = None

    def clear(self) -> None:
        self.bids.clear()
        self.asks.clear()
        self.sequence = 0
        self._last_sequence = None
        self.bids_truncated = False
        self.asks_truncated = False
