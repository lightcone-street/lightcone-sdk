"""Orders sub-client."""

from typing import Optional, TYPE_CHECKING
from urllib.parse import quote as url_quote

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
    GlobalDepositBalance,
    ConditionalBalance,
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
            remaining=data.get("remaining", 0),
        )

    async def cancel_all(self, body: CancelAllBody) -> CancelAllSuccess:
        """Cancel all orders for a user."""
        data = await self._http.post("/api/orders/cancel-all", body.to_dict())
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
        return TriggerOrderResponse(
            trigger_order_id=data.get("trigger_order_id", ""),
            order_hash=data.get("order_hash", ""),
        )

    async def cancel_trigger(self, body: CancelTriggerBody) -> CancelTriggerSuccess:
        """Cancel a trigger order."""
        data = await self._http.post("/api/orders/cancel", body.to_dict())
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

        orders = [
            UserSnapshotOrder(
                order_hash=o.get("order_hash", ""),
                side=o.get("side", 0),
                price=o.get("price", "0"),
                size=o.get("size", "0"),
                orderbook_id=o.get("orderbook_id", ""),
                market_pubkey=o.get("market_pubkey", ""),
                amount_in=o.get("amount_in", o.get("maker_amount", "0")),
                amount_out=o.get("amount_out", o.get("taker_amount", "0")),
                remaining=o.get("remaining", "0"),
                filled=o.get("filled", "0"),
                expiration=o.get("expiration", 0),
                base_mint=o.get("base_mint", ""),
                quote_mint=o.get("quote_mint", ""),
                outcome_index=o.get("outcome_index", 0),
                status=o.get("status", "open"),
                order_type=o.get("order_type", "limit"),
                created_at=o.get("created_at"),
                trigger_order_id=o.get("trigger_order_id"),
                trigger_price=o.get("trigger_price"),
                trigger_type=o.get("trigger_type"),
                time_in_force=o.get("time_in_force"),
            )
            for o in data.get("orders", [])
        ]

        balances = [
            UserSnapshotBalance(
                market_pubkey=b.get("market_pubkey", ""),
                orderbook_id=b.get("orderbook_id", ""),
                outcomes=[
                    ConditionalBalance(
                        outcome_index=c.get("outcome_index", 0),
                        mint=c.get("mint", c.get("conditional_token", "")),
                        idle=c.get("idle", "0"),
                        on_book=c.get("on_book", "0"),
                    )
                    for c in b.get("outcomes", [])
                ],
            )
            for b in data.get("balances", [])
        ]

        global_deposits = [
            GlobalDepositBalance(
                mint=g.get("mint", ""),
                balance=g.get("balance", "0"),
            )
            for g in data.get("global_deposits", [])
        ]

        return UserOrdersResponse(
            user_pubkey=data.get("user_pubkey", wallet),
            orders=orders,
            balances=balances,
            global_deposits=global_deposits,
            next_cursor=data.get("next_cursor"),
            has_more=data.get("has_more", False),
        )
