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
from ...error import SdkError
from ...shared.types import (
    SubmitOrderRequest,
    SubmitTriggerOrderRequest,
)

if TYPE_CHECKING:
    from ...http.client import LightconeHttp


class Orders:
    """Order operations sub-client."""

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def submit(self, request: SubmitOrderRequest) -> SubmitOrderResponse:
        """Submit a limit order."""
        data = await self._http.post("/api/orders/submit", request.to_dict())
        place = _unwrap_status(
            data,
            success_statuses={"accepted", "partial_fill", "filled"},
            rejected_statuses={"rejected"},
        )
        return submit_response_from_dict(place)

    async def cancel(self, body: CancelBody) -> CancelSuccess:
        """Cancel a single order."""
        data = await self._http.post("/api/orders/cancel", body.to_dict())
        data = _unwrap_status(data, success_statuses={"cancelled"})
        return CancelSuccess(
            order_hash=data.get("order_hash", body.order_hash),
            remaining=data.get("remaining", 0),
        )

    async def cancel_all(self, body: CancelAllBody) -> CancelAllSuccess:
        """Cancel all orders for a user."""
        data = await self._http.post("/api/orders/cancel-all", body.to_dict())
        data = _unwrap_status(data, success_statuses={"success"})
        return CancelAllSuccess(
            cancelled_order_hashes=data.get("cancelled_order_hashes", []),
            count=data.get("count", 0),
            user_pubkey=data.get("user_pubkey", body.user_pubkey),
            orderbook_id=data.get("orderbook_id", body.orderbook_id or ""),
            message=data.get("message", ""),
        )

    async def submit_trigger(self, request: SubmitTriggerOrderRequest) -> TriggerOrderResponse:
        """Submit a trigger order."""
        data = await self._http.post("/api/orders/submit", request.to_dict())
        data = _unwrap_status(data, success_statuses={"accepted"})
        return TriggerOrderResponse(
            trigger_order_id=data.get("trigger_order_id", ""),
            order_hash=data.get("order_hash", ""),
        )

    async def cancel_trigger(self, body: CancelTriggerBody) -> CancelTriggerSuccess:
        """Cancel a trigger order."""
        data = await self._http.post("/api/orders/cancel", body.to_dict())
        data = _unwrap_status(data, success_statuses={"cancelled"})
        return CancelTriggerSuccess(
            trigger_order_id=data.get("trigger_order_id", body.trigger_order_id),
        )

    async def get_user_orders(
        self,
        wallet: str,
        limit: Optional[int] = None,
        cursor: Optional[str] = None,
    ) -> UserOrdersResponse:
        """Get user's orders with pagination."""
        params: dict = {"wallet_address": wallet}
        if limit is not None:
            params["limit"] = str(limit)
        if cursor is not None:
            params["cursor"] = cursor

        data = await self._http.get("/api/users/orders", params=params)

        return UserOrdersResponse(
            user_pubkey=data.get("user_pubkey", wallet),
            orders=[UserSnapshotOrder.from_dict(o) for o in data.get("orders", [])],
            balances=[UserSnapshotBalance.from_dict(b) for b in data.get("balances", [])],
            next_cursor=data.get("next_cursor"),
            has_more=data.get("has_more", False),
        )


def _unwrap_status(
    data: dict,
    *,
    success_statuses: set[str],
    rejected_statuses: Optional[set[str]] = None,
) -> dict:
    status = data.get("status")
    if status is None or status in success_statuses:
        return data

    rejected_statuses = rejected_statuses or set()
    if status in rejected_statuses:
        error = data.get("error")
        details = data.get("details")
        if error and details:
            raise SdkError(f"{error}: {details}")
        if error:
            raise SdkError(error)
        if details:
            raise SdkError(details)

        parts = ["Rejected"]
        if data.get("order_hash"):
            parts.append(f"hash={data['order_hash']}")
        if data.get("filled") is not None:
            parts.append(f"filled={data['filled']}")
        if data.get("remaining") is not None:
            parts.append(f"remaining={data['remaining']}")
        raise SdkError(", ".join(parts))

    message = data.get("message")
    error = data.get("error")
    details = data.get("details")

    if error and details:
        raise SdkError(f"{error}: {details}")
    if error:
        raise SdkError(error)
    if message:
        raise SdkError(message)
    if details:
        raise SdkError(details)

    raise SdkError(f"Unexpected status: {status}")
