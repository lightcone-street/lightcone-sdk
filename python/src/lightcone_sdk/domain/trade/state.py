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
        """Insert a trade in ascending sequence order (oldest first, newest last).

        Trades with ``sequence > 0`` are placed at the correct position so the
        buffer stays sorted. Trades with ``sequence == 0`` (REST) are appended
        to the end as before.
        """
        # Treat a zero-capacity history as disabled.
        if self.max_size == 0:
            return
        if trade.sequence == 0:
            # REST trades do not carry ordering metadata, so preserve the legacy
            # append behavior and let deque(maxlen=...) handle capacity eviction.
            self._trades.append(trade)
            return
        # Default to "older than everything we currently keep". We scan from the
        # end because that is the newest side in this ascending buffer.
        insert_at = 0
        # Find the insertion point immediately after the newest trade with a
        # sequence less than or equal to the incoming one.
        for index in range(len(self._trades) - 1, -1, -1):
            if self._trades[index].sequence <= trade.sequence:
                insert_at = index + 1
                break
        # deque.insert() raises IndexError at maxlen, so only evict once we know
        # the new trade belongs in the retained window.
        if len(self._trades) >= self.max_size:
            if insert_at == 0:
                # The buffer is full and the new trade is older than everything
                # we already keep, so dropping it preserves the newest window.
                return
            # The new trade belongs in the retained window, so evict the oldest
            # trade from the front and adjust the insertion index.
            self._trades.popleft()
            insert_at -= 1
        self._trades.insert(insert_at, trade)

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
