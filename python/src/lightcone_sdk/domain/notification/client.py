"""Notifications sub-client."""

from __future__ import annotations

from typing import TYPE_CHECKING

from . import Notification, NotificationKind, MarketData, MarketResolvedData, OrderFilledData

if TYPE_CHECKING:
    from ...client import LightconeClient


class Notifications:
    """Notification operations sub-client."""

    def __init__(self, client: "LightconeClient"):
        self._client = client

    async def fetch(self) -> list[Notification]:
        """Get notifications for the authenticated user."""
        data = await self._client._http.get("/api/notifications")
        notifications_data = data.get("notifications", [])
        return [_parse_notification(n) for n in notifications_data]

    async def dismiss(self, notification_id: str) -> None:
        """Dismiss a notification."""
        await self._client._http.post("/api/notifications/dismiss", {
            "notification_id": notification_id,
        })


def _parse_notification(d: dict) -> Notification:
    """Parse a notification from API response."""
    kind_str = d.get("notification_type", "global")
    try:
        kind = NotificationKind(kind_str)
    except ValueError:
        kind = NotificationKind.GLOBAL

    notification = Notification(
        id=d.get("id", ""),
        kind=kind,
        title=d.get("title", ""),
        message=d.get("message", ""),
        expires_at=d.get("expires_at"),
        created_at=d.get("created_at"),
    )

    data = d.get("data", {})
    if kind == NotificationKind.MARKET_RESOLVED and data:
        notification.market_resolved_data = MarketResolvedData(
            market_pubkey=data.get("market_pubkey", ""),
            market_slug=data.get("market_slug"),
            market_name=data.get("market_name"),
            winning_outcome=data.get("winning_outcome"),
        )
    elif kind == NotificationKind.ORDER_FILLED and data:
        notification.order_filled_data = OrderFilledData(
            order_hash=data.get("order_hash", ""),
            market_pubkey=data.get("market_pubkey", ""),
            side=data.get("side", ""),
            price=data.get("price", "0"),
            filled=data.get("filled", "0"),
            remaining=data.get("remaining", "0"),
            market_slug=data.get("market_slug"),
            market_name=data.get("market_name"),
            outcome_name=data.get("outcome_name"),
            outcome_icon_url=data.get("outcome_icon_url"),
        )
    elif kind in (NotificationKind.NEW_MARKET, NotificationKind.RULES_CLARIFIED) and data:
        notification.market_data = MarketData(
            market_pubkey=data.get("market_pubkey", ""),
            market_slug=data.get("market_slug"),
            market_name=data.get("market_name"),
        )

    return notification
