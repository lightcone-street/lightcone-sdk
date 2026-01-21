"""Subscription management for WebSocket channels."""

from dataclasses import dataclass, field
from typing import Optional

from .types import (
    book_update_params,
    trades_params,
    user_params,
    price_history_params,
    market_params,
)


@dataclass
class Subscription:
    """Represents a subscription to a specific channel."""

    type_: str
    orderbook_ids: Optional[list[str]] = None
    user: Optional[str] = None
    orderbook_id: Optional[str] = None
    resolution: Optional[str] = None
    include_ohlcv: Optional[bool] = None
    market_pubkey: Optional[str] = None

    def to_params(self) -> dict:
        """Convert to subscription parameters for sending."""
        if self.type_ == "book_update" and self.orderbook_ids:
            return book_update_params(self.orderbook_ids)
        elif self.type_ == "trades" and self.orderbook_ids:
            return trades_params(self.orderbook_ids)
        elif self.type_ == "user" and self.user:
            return user_params(self.user)
        elif self.type_ == "price_history" and self.orderbook_id and self.resolution:
            return price_history_params(
                self.orderbook_id,
                self.resolution,
                self.include_ohlcv or False,
            )
        elif self.type_ == "market" and self.market_pubkey:
            return market_params(self.market_pubkey)
        else:
            raise ValueError(f"Invalid subscription: {self}")


class SubscriptionManager:
    """Manages active subscriptions."""

    def __init__(self):
        self._book_updates: set[str] = set()
        self._trades: set[str] = set()
        self._users: set[str] = set()
        # key -> (orderbook_id, resolution, include_ohlcv)
        self._price_history: dict[str, tuple[str, str, bool]] = {}
        self._markets: set[str] = set()

    def add_book_update(self, orderbook_ids: list[str]) -> None:
        """Add a book update subscription."""
        for id_ in orderbook_ids:
            self._book_updates.add(id_)

    def remove_book_update(self, orderbook_ids: list[str]) -> None:
        """Remove a book update subscription."""
        for id_ in orderbook_ids:
            self._book_updates.discard(id_)

    def is_subscribed_book_update(self, orderbook_id: str) -> bool:
        """Check if subscribed to book updates for an orderbook."""
        return orderbook_id in self._book_updates

    def add_trades(self, orderbook_ids: list[str]) -> None:
        """Add a trades subscription."""
        for id_ in orderbook_ids:
            self._trades.add(id_)

    def remove_trades(self, orderbook_ids: list[str]) -> None:
        """Remove a trades subscription."""
        for id_ in orderbook_ids:
            self._trades.discard(id_)

    def is_subscribed_trades(self, orderbook_id: str) -> bool:
        """Check if subscribed to trades for an orderbook."""
        return orderbook_id in self._trades

    def add_user(self, user: str) -> None:
        """Add a user subscription."""
        self._users.add(user)

    def remove_user(self, user: str) -> None:
        """Remove a user subscription."""
        self._users.discard(user)

    def is_subscribed_user(self, user: str) -> bool:
        """Check if subscribed to a user."""
        return user in self._users

    def add_price_history(
        self,
        orderbook_id: str,
        resolution: str,
        include_ohlcv: bool,
    ) -> None:
        """Add a price history subscription."""
        key = f"{orderbook_id}:{resolution}"
        self._price_history[key] = (orderbook_id, resolution, include_ohlcv)

    def remove_price_history(self, orderbook_id: str, resolution: str) -> None:
        """Remove a price history subscription."""
        key = f"{orderbook_id}:{resolution}"
        self._price_history.pop(key, None)

    def is_subscribed_price_history(self, orderbook_id: str, resolution: str) -> bool:
        """Check if subscribed to price history for an orderbook/resolution."""
        key = f"{orderbook_id}:{resolution}"
        return key in self._price_history

    def add_market(self, market_pubkey: str) -> None:
        """Add a market subscription."""
        self._markets.add(market_pubkey)

    def remove_market(self, market_pubkey: str) -> None:
        """Remove a market subscription."""
        self._markets.discard(market_pubkey)

    def is_subscribed_market(self, market_pubkey: str) -> bool:
        """Check if subscribed to market events."""
        return market_pubkey in self._markets or "all" in self._markets

    def get_all_subscriptions(self) -> list[Subscription]:
        """Get all subscriptions for re-subscribing after reconnect."""
        subs = []

        # Group book updates
        if self._book_updates:
            subs.append(
                Subscription(
                    type_="book_update",
                    orderbook_ids=list(self._book_updates),
                )
            )

        # Group trades
        if self._trades:
            subs.append(
                Subscription(
                    type_="trades",
                    orderbook_ids=list(self._trades),
                )
            )

        # Users
        for user in self._users:
            subs.append(Subscription(type_="user", user=user))

        # Price history
        for orderbook_id, resolution, include_ohlcv in self._price_history.values():
            subs.append(
                Subscription(
                    type_="price_history",
                    orderbook_id=orderbook_id,
                    resolution=resolution,
                    include_ohlcv=include_ohlcv,
                )
            )

        # Markets
        for market_pubkey in self._markets:
            subs.append(Subscription(type_="market", market_pubkey=market_pubkey))

        return subs

    def clear(self) -> None:
        """Clear all subscriptions."""
        self._book_updates.clear()
        self._trades.clear()
        self._users.clear()
        self._price_history.clear()
        self._markets.clear()

    def has_subscriptions(self) -> bool:
        """Check if there are any active subscriptions."""
        return bool(
            self._book_updates
            or self._trades
            or self._users
            or self._price_history
            or self._markets
        )

    def subscription_count(self) -> int:
        """Get count of active subscriptions."""
        return (
            len(self._book_updates)
            + len(self._trades)
            + len(self._users)
            + len(self._price_history)
            + len(self._markets)
        )

    def book_update_orderbooks(self) -> list[str]:
        """Get all subscribed orderbook IDs (for book updates)."""
        return list(self._book_updates)

    def trade_orderbooks(self) -> list[str]:
        """Get all subscribed orderbook IDs (for trades)."""
        return list(self._trades)

    def subscribed_users(self) -> list[str]:
        """Get all subscribed users."""
        return list(self._users)

    def subscribed_markets(self) -> list[str]:
        """Get all subscribed markets."""
        return list(self._markets)
