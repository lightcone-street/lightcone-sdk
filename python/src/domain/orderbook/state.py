"""Orderbook state for WebSocket updates.

Refactored from websocket/state/orderbook.py.
"""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class OrderbookSnapshot:
    """Local orderbook state maintained from WebSocket updates."""
    orderbook_id: str
    bids: dict[str, str] = field(default_factory=dict)
    asks: dict[str, str] = field(default_factory=dict)
    sequence: int = 0

    def apply(self, update: dict) -> None:
        """Apply a book update (snapshot or delta)."""
        is_snapshot = update.get("is_snapshot", False)

        if is_snapshot:
            self.bids.clear()
            self.asks.clear()

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

    def best_bid(self) -> Optional[str]:
        if not self.bids:
            return None
        return max(self.bids.keys(), key=lambda p: float(p))

    def best_ask(self) -> Optional[str]:
        if not self.asks:
            return None
        return min(self.asks.keys(), key=lambda p: float(p))

    def mid_price(self) -> Optional[float]:
        bb = self.best_bid()
        ba = self.best_ask()
        if bb is None or ba is None:
            return None
        return (float(bb) + float(ba)) / 2

    def spread(self) -> Optional[float]:
        bb = self.best_bid()
        ba = self.best_ask()
        if bb is None or ba is None:
            return None
        return float(ba) - float(bb)

    def is_empty(self) -> bool:
        return not self.bids and not self.asks

    def clear(self) -> None:
        self.bids.clear()
        self.asks.clear()
        self.sequence = 0
