"""User state management."""

from decimal import Decimal, InvalidOperation
from typing import Optional

from ..types import Balance, BalanceEntry, Order, UserEventData


def is_zero(s: str) -> bool:
    """Check if a string represents zero with decimal precision."""
    try:
        return Decimal(s) == 0
    except InvalidOperation:
        return False


class UserState:
    """User state tracking orders and balances."""

    def __init__(self, user: str):
        self.user = user
        self._orders: dict[str, Order] = {}  # order_hash -> Order
        self._balances: dict[str, BalanceEntry] = {}  # market:deposit_mint -> BalanceEntry
        self._has_snapshot: bool = False
        self._last_timestamp: Optional[str] = None

    def apply_snapshot(self, data: UserEventData) -> None:
        """Apply a snapshot (full user state)."""
        self._orders.clear()
        self._balances.clear()

        for order in data.orders:
            self._orders[order.order_hash] = order

        for key, balance in data.balances.items():
            self._balances[key] = balance

        self._has_snapshot = True
        self._last_timestamp = data.timestamp

    def apply_order_update(self, data: UserEventData) -> None:
        """Apply an order update."""
        if data.order is None:
            return

        update = data.order
        order_hash = update.order_hash

        # If remaining is 0, the order is fully filled or cancelled - remove it
        if is_zero(update.remaining):
            self._orders.pop(order_hash, None)
        elif order_hash in self._orders:
            # Update existing order
            existing = self._orders[order_hash]
            existing.remaining = update.remaining
            existing.filled = update.filled
        else:
            # New order - construct from update
            if data.market_pubkey and data.orderbook_id:
                order = Order(
                    order_hash=order_hash,
                    market_pubkey=data.market_pubkey,
                    orderbook_id=data.orderbook_id,
                    side=update.side,
                    maker_amount=update.remaining,
                    taker_amount="0",
                    remaining=update.remaining,
                    filled=update.filled,
                    price=update.price,
                    created_at=update.created_at,
                    expiration=0,
                )
                self._orders[order_hash] = order

        # Apply balance updates if present
        if update.balance:
            self._apply_balance_from_order(data, update.balance)

        self._last_timestamp = data.timestamp

    def apply_balance_update(self, data: UserEventData) -> None:
        """Apply a balance update."""
        if data.market_pubkey and data.deposit_mint and data.balance:
            key = f"{data.market_pubkey}:{data.deposit_mint}"
            entry = BalanceEntry(
                market_pubkey=data.market_pubkey,
                deposit_mint=data.deposit_mint,
                outcomes=data.balance.outcomes,
            )
            self._balances[key] = entry

        self._last_timestamp = data.timestamp

    def _apply_balance_from_order(self, data: UserEventData, balance: Balance) -> None:
        """Apply balance from order update."""
        if data.market_pubkey and data.deposit_mint:
            key = f"{data.market_pubkey}:{data.deposit_mint}"
            entry = BalanceEntry(
                market_pubkey=data.market_pubkey,
                deposit_mint=data.deposit_mint,
                outcomes=balance.outcomes,
            )
            self._balances[key] = entry
        elif data.market_pubkey:
            # If no deposit_mint, update existing entry with matching market
            for key, entry in self._balances.items():
                if key.startswith(data.market_pubkey):
                    entry.outcomes = balance.outcomes
                    break

    def apply_event(self, data: UserEventData) -> None:
        """Apply any user event."""
        if data.event_type == "snapshot":
            self.apply_snapshot(data)
        elif data.event_type == "order_update":
            self.apply_order_update(data)
        elif data.event_type == "balance_update":
            self.apply_balance_update(data)

    def get_order(self, order_hash: str) -> Optional[Order]:
        """Get an order by hash."""
        return self._orders.get(order_hash)

    def open_orders(self) -> list[Order]:
        """Get all open orders."""
        return list(self._orders.values())

    def orders_for_market(self, market_pubkey: str) -> list[Order]:
        """Get orders for a specific market."""
        return [o for o in self._orders.values() if o.market_pubkey == market_pubkey]

    def orders_for_orderbook(self, orderbook_id: str) -> list[Order]:
        """Get orders for a specific orderbook."""
        return [o for o in self._orders.values() if o.orderbook_id == orderbook_id]

    def get_balance(self, market_pubkey: str, deposit_mint: str) -> Optional[BalanceEntry]:
        """Get balance for a market/deposit_mint pair."""
        key = f"{market_pubkey}:{deposit_mint}"
        return self._balances.get(key)

    def all_balances(self) -> list[BalanceEntry]:
        """Get all balances."""
        return list(self._balances.values())

    def idle_balance_for_outcome(
        self,
        market_pubkey: str,
        deposit_mint: str,
        outcome_index: int,
    ) -> Optional[str]:
        """Get total idle balance for a specific outcome."""
        balance = self.get_balance(market_pubkey, deposit_mint)
        if balance:
            for outcome in balance.outcomes:
                if outcome.outcome_index == outcome_index:
                    return outcome.idle
        return None

    def on_book_balance_for_outcome(
        self,
        market_pubkey: str,
        deposit_mint: str,
        outcome_index: int,
    ) -> Optional[str]:
        """Get total on-book balance for a specific outcome."""
        balance = self.get_balance(market_pubkey, deposit_mint)
        if balance:
            for outcome in balance.outcomes:
                if outcome.outcome_index == outcome_index:
                    return outcome.on_book
        return None

    def order_count(self) -> int:
        """Number of open orders."""
        return len(self._orders)

    def has_snapshot(self) -> bool:
        """Whether the user state has received its initial snapshot."""
        return self._has_snapshot

    def last_timestamp(self) -> Optional[str]:
        """Last update timestamp."""
        return self._last_timestamp

    def clear(self) -> None:
        """Clear the user state (for disconnect/resync)."""
        self._orders.clear()
        self._balances.clear()
        self._has_snapshot = False
        self._last_timestamp = None
