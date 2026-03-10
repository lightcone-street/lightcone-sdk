"""Trade history state (ring buffer)."""

from collections import deque
from dataclasses import dataclass, field
from typing import Optional
from . import Trade


@dataclass
class TradeHistory:
    """Ring buffer of recent trades."""
    max_size: int = 100
    _trades: deque = field(default_factory=lambda: deque(maxlen=100))

    def __post_init__(self):
        self._trades = deque(maxlen=self.max_size)

    def add(self, trade: Trade) -> None:
        self._trades.append(trade)

    def all(self) -> list[Trade]:
        return list(self._trades)

    def latest(self) -> Optional[Trade]:
        return self._trades[-1] if self._trades else None

    def clear(self) -> None:
        self._trades.clear()
