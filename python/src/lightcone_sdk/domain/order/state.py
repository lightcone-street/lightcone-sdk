"""Order state containers for WebSocket updates."""

from dataclasses import dataclass, field
from typing import Optional
from . import Order, TriggerOrder


@dataclass
class UserOpenOrders:
    """Container for a user's open orders, grouped by market_pubkey."""
    orders: dict[str, list[Order]] = field(default_factory=dict)

    def upsert(self, order: Order) -> None:
        market = order.market_pubkey
        if market not in self.orders:
            self.orders[market] = []
        lst = self.orders[market]
        # Remove existing order with same hash before appending
        self.orders[market] = [o for o in lst if o.order_hash != order.order_hash]
        self.orders[market].append(order)

    def remove(self, order_hash: str) -> Optional[Order]:
        for market, lst in self.orders.items():
            for i, o in enumerate(lst):
                if o.order_hash == order_hash:
                    return lst.pop(i)
        return None

    def update(self, order: Order) -> None:
        if order.status in ("cancelled", "filled"):
            self.remove(order.order_hash)
        else:
            self.upsert(order)

    def get(self, market_pubkey: str) -> Optional[list[Order]]:
        return self.orders.get(market_pubkey)

    def all(self) -> list[Order]:
        result = []
        for lst in self.orders.values():
            result.extend(lst)
        return result

    def is_empty(self) -> bool:
        return all(len(v) == 0 for v in self.orders.values())

    def clear(self) -> None:
        self.orders.clear()


@dataclass
class UserTriggerOrders:
    """Container for a user's trigger orders, keyed by orderbook_id."""
    orders: dict[str, list[TriggerOrder]] = field(default_factory=dict)

    def insert(self, order: TriggerOrder) -> None:
        key = order.orderbook_id
        if key not in self.orders:
            self.orders[key] = []
        self.orders[key].append(order)

    def remove(self, trigger_id: str) -> Optional[TriggerOrder]:
        for key, order_list in self.orders.items():
            for i, o in enumerate(order_list):
                if o.trigger_order_id == trigger_id:
                    return order_list.pop(i)
        return None

    def get(self, orderbook_id: str) -> Optional[list[TriggerOrder]]:
        return self.orders.get(orderbook_id)

    def get_by_id(self, trigger_id: str) -> Optional[TriggerOrder]:
        for order_list in self.orders.values():
            for o in order_list:
                if o.trigger_order_id == trigger_id:
                    return o
        return None

    def all(self) -> list[TriggerOrder]:
        result = []
        for order_list in self.orders.values():
            result.extend(order_list)
        return result

    def is_empty(self) -> bool:
        return all(len(v) == 0 for v in self.orders.values())

    def __len__(self) -> int:
        return sum(len(v) for v in self.orders.values())

    def clear(self) -> None:
        self.orders.clear()
