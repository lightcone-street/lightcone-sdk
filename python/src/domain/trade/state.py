"""Trade history state (ring buffer)."""

from collections import deque
from dataclasses import dataclass, field
from typing import Optional
from . import Trade


@dataclass
class TradeHistory:
    """Ring buffer of recent trades."""
    orderbook_id: str = ""
    max_size: int = 100
    _trades: deque = field(default_factory=lambda: deque(maxlen=100))

    def __post_init__(self):
        self._trades = deque(maxlen=self.max_size)

    def push(self, trade: Trade) -> None:
        self._trades.append(trade)

    def add(self, trade: Trade) -> None:
        """Alias for push()."""
        self.push(trade)

    def replace(self, trades: list[Trade]) -> None:
        """Replace all trades."""
        self._trades.clear()
        for t in trades:
            self._trades.append(t)

    def trades(self) -> list[Trade]:
        return list(self._trades)

    def all(self) -> list[Trade]:
        """Alias for trades()."""
        return self.trades()

    def latest(self) -> Optional[Trade]:
        return self._trades[-1] if self._trades else None

    def __len__(self) -> int:
        return len(self._trades)

    def is_empty(self) -> bool:
        return len(self._trades) == 0

    def clear(self) -> None:
        self._trades.clear()
