"""Order state containers for WebSocket updates."""

from dataclasses import dataclass, field
from typing import Optional
from . import Order, TriggerOrder


@dataclass
class UserOpenOrders:
    """Container for a user's open orders."""
    orders: dict[str, Order] = field(default_factory=dict)

    def update(self, order: Order) -> None:
        if order.status in ("cancelled", "filled"):
            self.orders.pop(order.order_hash, None)
        else:
            self.orders[order.order_hash] = order

    def get(self, order_hash: str) -> Optional[Order]:
        return self.orders.get(order_hash)

    def all(self) -> list[Order]:
        return list(self.orders.values())

    def clear(self) -> None:
        self.orders.clear()


@dataclass
class UserTriggerOrders:
    """Container for a user's trigger orders."""
    orders: dict[str, TriggerOrder] = field(default_factory=dict)

    def update(self, order: TriggerOrder) -> None:
        if order.status in ("cancelled", "triggered", "expired", "failed"):
            self.orders.pop(order.trigger_order_id, None)
        else:
            self.orders[order.trigger_order_id] = order

    def get(self, trigger_id: str) -> Optional[TriggerOrder]:
        return self.orders.get(trigger_id)

    def all(self) -> list[TriggerOrder]:
        return list(self.orders.values())

    def clear(self) -> None:
        self.orders.clear()
