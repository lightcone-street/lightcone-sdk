"""Orderbook state for WebSocket updates.

The ``book_update`` stream is snapshot-only: every data frame carries the
full top-20 levels per side and replaces the previous book wholesale
(last-write-wins). Consumers holding multiple aggregation views of one
orderbook on the same connection key their :class:`OrderbookState` instances
by ``(orderbook_id, aggregation)`` using ``WsOrderBook.aggregation()``.
"""

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Optional


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


@dataclass
class OrderbookState:
    """Local orderbook state replaced wholesale by snapshot frames.

    ``sequence`` is the projection version of the last applied frame:
    strictly increasing but non-contiguous server-side (conflation skips
    versions), and the initial snapshot after every (re)subscribe is
    ``seq: 0`` — informational only, never used to gate frames.
    """

    orderbook_id: str
    bids: dict[str, str] = field(default_factory=dict)
    asks: dict[str, str] = field(default_factory=dict)
    sequence: int = 0

    def apply(self, update) -> OrderbookApplyResult:
        """Apply a WS orderbook frame (snapshot-only stream, last-write-wins).

        Accepts either a raw dict or a WsOrderBook dataclass. ``resync``
        frames take precedence and leave the book untouched — the caller must
        re-subscribe with the same parameters. Every other data frame is a
        full snapshot by contract and replaces the book wholesale (the
        ``is_snapshot`` flag is not consulted), including the ``seq: 0``
        initial snapshot delivered after every (re)subscribe: gating on
        ``seq`` would freeze the book after a resync or aggregation change.
        """
        if hasattr(update, "is_snapshot"):
            return self._apply_typed(update)
        return self._apply_dict(update)

    def _apply_typed(self, update) -> OrderbookApplyResult:
        if getattr(update, "resync", False):
            return OrderbookApplyResult.refresh_required(
                OrderbookRefreshReason.server_resync()
            )

        self.bids.clear()
        self.asks.clear()
        for bid in update.bids:
            if bid.size != "0":
                self.bids[bid.price] = bid.size
        for ask in update.asks:
            if ask.size != "0":
                self.asks[ask.price] = ask.size
        self.sequence = update.seq
        return OrderbookApplyResult.applied()

    def _apply_dict(self, update: dict) -> OrderbookApplyResult:
        if update.get("resync", False):
            return OrderbookApplyResult.refresh_required(
                OrderbookRefreshReason.server_resync()
            )

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

        self.sequence = update.get("seq", 0)
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
