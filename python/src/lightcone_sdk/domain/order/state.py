"""Order state containers for WebSocket updates."""

from dataclasses import dataclass, field
from typing import Optional
from . import LimitOrder, OrderStatus, TriggerOrder


@dataclass
class UserOpenLimitOrders:
    """Container for a user's open limit orders, grouped by market_pubkey -> orderbook_id."""
    orders: dict[str, dict[str, list[LimitOrder]]] = field(default_factory=dict)

    def upsert(self, order: LimitOrder) -> None:
        market = order.market_pubkey
        orderbook = order.orderbook_id
        if market not in self.orders:
            self.orders[market] = {}
        if orderbook not in self.orders[market]:
            self.orders[market][orderbook] = []
        order_list = self.orders[market][orderbook]
        # Remove existing order with same hash before appending
        self.orders[market][orderbook] = [
            existing for existing in order_list if existing.order_hash != order.order_hash
        ]
        self.orders[market][orderbook].append(order)

    def remove(self, order_hash: str) -> Optional[LimitOrder]:
        for market_orders in self.orders.values():
            for orderbook_id, order_list in market_orders.items():
                for index, existing in enumerate(order_list):
                    if existing.order_hash == order_hash:
                        return order_list.pop(index)
        return None

    def update(self, order: LimitOrder) -> None:
        if order.status in (OrderStatus.CANCELLED, OrderStatus.FILLED):
            self.remove(order.order_hash)
        else:
            self.upsert(order)

    def get(self, market_pubkey: str, orderbook_id: str) -> Optional[list[LimitOrder]]:
        market_orders = self.orders.get(market_pubkey)
        if market_orders is None:
            return None
        return market_orders.get(orderbook_id)

    def get_by_market(self, market_pubkey: str) -> Optional[dict[str, list[LimitOrder]]]:
        return self.orders.get(market_pubkey)

    def all(self) -> list[LimitOrder]:
        result = []
        for market_orders in self.orders.values():
            for order_list in market_orders.values():
                result.extend(order_list)
        return result

    def is_empty(self) -> bool:
        return all(
            len(order_list) == 0
            for market_orders in self.orders.values()
            for order_list in market_orders.values()
        )

    def clear(self) -> None:
        self.orders.clear()


@dataclass
class UserTriggerOrders:
    """Container for a user's trigger orders, grouped by market_pubkey -> orderbook_id."""
    orders: dict[str, dict[str, list[TriggerOrder]]] = field(default_factory=dict)

    def insert(self, order: TriggerOrder) -> None:
        market = order.market_pubkey
        orderbook = order.orderbook_id
        if market not in self.orders:
            self.orders[market] = {}
        if orderbook not in self.orders[market]:
            self.orders[market][orderbook] = []
        self.orders[market][orderbook].append(order)

    def remove(self, trigger_id: str) -> Optional[TriggerOrder]:
        for market_orders in self.orders.values():
            for orderbook_id, order_list in market_orders.items():
                for index, existing in enumerate(order_list):
                    if existing.trigger_order_id == trigger_id:
                        return order_list.pop(index)
        return None

    def get(self, market_pubkey: str, orderbook_id: str) -> Optional[list[TriggerOrder]]:
        market_orders = self.orders.get(market_pubkey)
        if market_orders is None:
            return None
        return market_orders.get(orderbook_id)

    def get_by_market(self, market_pubkey: str) -> Optional[dict[str, list[TriggerOrder]]]:
        return self.orders.get(market_pubkey)

    def get_by_id(self, trigger_id: str) -> Optional[TriggerOrder]:
        for market_orders in self.orders.values():
            for order_list in market_orders.values():
                for existing in order_list:
                    if existing.trigger_order_id == trigger_id:
                        return existing
        return None

    def all(self) -> list[TriggerOrder]:
        result = []
        for market_orders in self.orders.values():
            for order_list in market_orders.values():
                result.extend(order_list)
        return result

    def is_empty(self) -> bool:
        return all(
            len(order_list) == 0
            for market_orders in self.orders.values()
            for order_list in market_orders.values()
        )

    def __len__(self) -> int:
        return sum(
            len(order_list)
            for market_orders in self.orders.values()
            for order_list in market_orders.values()
        )

    def clear(self) -> None:
        self.orders.clear()
