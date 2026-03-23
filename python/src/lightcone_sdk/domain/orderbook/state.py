"""Orderbook state for WebSocket updates."""

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Optional, Union


@dataclass
class OrderbookSnapshot:
    """Local orderbook state maintained from WebSocket updates.

    Stores bids and asks as ``Decimal`` → ``Decimal`` mappings (price → size)
    for exact arithmetic. Matches Rust's ``BTreeMap<Decimal, Decimal>`` pattern.
    """
    orderbook_id: str
    bids: dict[Decimal, Decimal] = field(default_factory=dict)
    asks: dict[Decimal, Decimal] = field(default_factory=dict)
    sequence: int = 0

    def apply(self, update) -> None:
        """Apply a book update (snapshot or delta).

        Accepts either a raw dict or a WsOrderBook dataclass.
        """
        if hasattr(update, "is_snapshot"):
            self._apply_typed(update)
        else:
            self._apply_dict(update)

    def _apply_typed(self, update) -> None:
        """Apply a WsOrderBook dataclass."""
        if update.is_snapshot:
            self.bids.clear()
            self.asks.clear()

        for bid in update.bids:
            price = Decimal(bid.price)
            size = Decimal(bid.size)
            if size == 0:
                self.bids.pop(price, None)
            else:
                self.bids[price] = size

        for ask in update.asks:
            price = Decimal(ask.price)
            size = Decimal(ask.size)
            if size == 0:
                self.asks.pop(price, None)
            else:
                self.asks[price] = size

        self.sequence = update.seq

    def _apply_dict(self, update: dict) -> None:
        """Apply a raw dict update."""
        if update.get("is_snapshot", False):
            self.bids.clear()
            self.asks.clear()

        for bid in update.get("bids", []):
            price = Decimal(str(bid.get("price", bid[0] if isinstance(bid, list) else "0")))
            size = Decimal(str(bid.get("size", bid[1] if isinstance(bid, list) and len(bid) > 1 else "0")))
            if size == 0:
                self.bids.pop(price, None)
            else:
                self.bids[price] = size

        for ask in update.get("asks", []):
            price = Decimal(str(ask.get("price", ask[0] if isinstance(ask, list) else "0")))
            size = Decimal(str(ask.get("size", ask[1] if isinstance(ask, list) and len(ask) > 1 else "0")))
            if size == 0:
                self.asks.pop(price, None)
            else:
                self.asks[price] = size

        seq = update.get("seq")
        if seq is not None:
            self.sequence = seq

    def best_bid(self) -> Optional[str]:
        """Highest bid price, or None if no bids."""
        if not self.bids:
            return None
        return str(max(self.bids.keys()))

    def best_ask(self) -> Optional[str]:
        """Lowest ask price, or None if no asks."""
        if not self.asks:
            return None
        return str(min(self.asks.keys()))

    def mid_price(self) -> Optional[str]:
        """Average of best bid and best ask, or None."""
        bb = self.best_bid()
        ba = self.best_ask()
        if bb is None or ba is None:
            return None
        return str((Decimal(bb) + Decimal(ba)) / 2)

    def spread(self) -> Optional[str]:
        """Difference between best ask and best bid, or None."""
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
