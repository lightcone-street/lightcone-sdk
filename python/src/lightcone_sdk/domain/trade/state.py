"""Trade history state — app-owned, SDK-provided update logic."""

from __future__ import annotations

import dataclasses

from . import Trade


class TradeHistory:
    """Rolling trade history buffer for an orderbook, newest first.

    The app owns instances of this type. The SDK provides update methods. The
    history stores copies and never changes a pushed :class:`Trade`.
    """

    def __init__(self, orderbook_id: str = "", max_size: int = 100):
        self.orderbook_id = orderbook_id
        self.max_size = max_size
        self._trades: list[Trade] = []

    def push(self, trade: Trade) -> None:
        """Push a trade, keeping the buffer newest-first by execution time.

        Trades are deduplicated by ``trade_id``, which REST and WS share, so
        live trades can be merged into REST history (a duplicate only fills in
        a missing REST ``cursor_id``). Equal timestamps keep arrival order,
        newest first. When full, a trade older than everything retained is
        dropped.
        """
        # Treat a zero-capacity history as disabled.
        if self.max_size <= 0:
            return
        for index, existing in enumerate(self._trades):
            if existing.trade_id == trade.trade_id:
                if existing.cursor_id is None and trade.cursor_id is not None:
                    self._trades[index] = dataclasses.replace(
                        existing, cursor_id=trade.cursor_id
                    )
                return
        position = next(
            (
                index
                for index, existing in enumerate(self._trades)
                if existing.timestamp <= trade.timestamp
            ),
            len(self._trades),
        )
        if len(self._trades) >= self.max_size and position == len(self._trades):
            return
        self._trades.insert(position, dataclasses.replace(trade))
        if len(self._trades) > self.max_size:
            self._trades.pop()

    def add(self, trade: Trade) -> None:
        """Alias for :meth:`push`."""
        self.push(trade)

    def replace(self, trades: list[Trade]) -> None:
        """Replace all trades (e.g. from a REST fetch), keeping at most ``max_size``."""
        self._trades = [
            dataclasses.replace(trade) for trade in trades[: max(self.max_size, 0)]
        ]

    def trades(self) -> list[Trade]:
        """The retained trades, newest first."""
        return list(self._trades)

    def all(self) -> list[Trade]:
        """Alias for :meth:`trades`."""
        return self.trades()

    def latest(self) -> Trade | None:
        return self._trades[0] if self._trades else None

    def __len__(self) -> int:
        return len(self._trades)

    def is_empty(self) -> bool:
        return not self._trades

    def clear(self) -> None:
        self._trades.clear()


__all__ = ["TradeHistory"]
