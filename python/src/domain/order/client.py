"""Orders sub-client."""

from typing import Optional, TYPE_CHECKING

from . import (
    SubmitOrderResponse,
    CancelBody,
    CancelSuccess,
    CancelAllBody,
    CancelAllSuccess,
    CancelTriggerBody,
    CancelTriggerSuccess,
    TriggerOrderResponse,
    UserOrdersResponse,
    UserSnapshotOrder,
    UserSnapshotBalance,
)
from .convert import submit_response_from_dict
from ...shared.types import SubmitOrderRequest, SubmitTriggerOrderRequest

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class Orders:
    """Order operations sub-client."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def submit(self, request: SubmitOrderRequest) -> SubmitOrderResponse:
        """Submit a limit order."""
        data = await self._http.post("/api/orders/submit", request.to_dict())
        return submit_response_from_dict(data)

    async def cancel(self, body: CancelBody) -> CancelSuccess:
        """Cancel a single order."""
        data = await self._http.post("/api/orders/cancel", body.to_dict())
        return CancelSuccess(
            order_hash=data.get("order_hash", body.order_hash),
            success=data.get("success", True),
        )

    async def cancel_all(self, body: CancelAllBody) -> CancelAllSuccess:
        """Cancel all orders for a user."""
        data = await self._http.post("/api/orders/cancel-all", body.to_dict())
        return CancelAllSuccess(
            cancelled=data.get("cancelled", []),
            success=data.get("success", True),
        )

    async def submit_trigger(self, request: SubmitTriggerOrderRequest) -> TriggerOrderResponse:
        """Submit a trigger order."""
        data = await self._http.post("/api/orders/submit-trigger", request.to_dict())
        return TriggerOrderResponse(
            trigger_order_id=data.get("trigger_order_id", ""),
            success=data.get("success", True),
            message=data.get("message"),
        )

    async def cancel_trigger(self, body: CancelTriggerBody) -> CancelTriggerSuccess:
        """Cancel a trigger order."""
        data = await self._http.post("/api/orders/cancel-trigger", body.to_dict())
        return CancelTriggerSuccess(
            trigger_order_id=data.get("trigger_order_id", body.trigger_order_id),
            success=data.get("success", True),
        )

    async def get_user_orders(
        self,
        wallet: str,
        limit: Optional[int] = None,
        cursor: Optional[str] = None,
    ) -> UserOrdersResponse:
        """Get user's orders with pagination."""
        body: dict = {"user_pubkey": wallet}
        if limit is not None:
            body["limit"] = limit
        if cursor is not None:
            body["cursor"] = cursor

        data = await self._http.post("/api/users/orders", body)

        orders = [
            UserSnapshotOrder(
                order_hash=o.get("order_hash", ""),
                side=o.get("side", 0),
                price=o.get("price", "0"),
                size=o.get("size", "0"),
                orderbook_id=o.get("orderbook_id", ""),
                status=o.get("status", "open"),
                order_type=o.get("order_type", "limit"),
                trigger_order_id=o.get("trigger_order_id"),
            )
            for o in data.get("orders", [])
        ]

        balances = [
            UserSnapshotBalance(
                market_pubkey=b.get("market_pubkey", ""),
            )
            for b in data.get("balances", [])
        ]

        return UserOrdersResponse(
            orders=orders,
            balances=balances,
            next_cursor=data.get("next_cursor"),
        )
