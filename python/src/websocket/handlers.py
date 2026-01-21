"""Message handlers for WebSocket events."""

import json
import logging
from typing import Optional

from .error import MessageParseError, SequenceGapError
from .state import LocalOrderbook, PriceHistory, PriceHistoryKey, UserState
from .types import (
    BookUpdateData,
    ErrorData,
    MarketEventData,
    MessageType,
    PriceHistoryData,
    RawWsMessage,
    TradeData,
    UserEventData,
    WsEvent,
)

logger = logging.getLogger(__name__)


class MessageHandler:
    """Handles incoming WebSocket messages."""

    def __init__(self):
        self._orderbooks: dict[str, LocalOrderbook] = {}
        self._user_states: dict[str, UserState] = {}
        self._price_histories: dict[PriceHistoryKey, PriceHistory] = {}
        self._subscribed_user: Optional[str] = None

    async def handle_message(self, text: str) -> list[WsEvent]:
        """Handle an incoming message and return events."""
        try:
            data = json.loads(text)
            raw_msg = RawWsMessage.from_dict(data)
        except json.JSONDecodeError as e:
            logger.warning(f"Failed to parse WebSocket message: {e}")
            return [WsEvent.error(MessageParseError(str(e)))]

        msg_type = MessageType.from_str(raw_msg.type_)

        if msg_type == MessageType.BOOK_UPDATE:
            return self._handle_book_update(raw_msg)
        elif msg_type == MessageType.TRADES:
            return self._handle_trade(raw_msg)
        elif msg_type == MessageType.USER:
            return self._handle_user_event(raw_msg)
        elif msg_type == MessageType.PRICE_HISTORY:
            return self._handle_price_history(raw_msg)
        elif msg_type == MessageType.MARKET:
            return self._handle_market_event(raw_msg)
        elif msg_type == MessageType.ERROR:
            return self._handle_error(raw_msg)
        elif msg_type == MessageType.PONG:
            return [WsEvent.pong()]
        else:
            logger.warning(f"Unknown message type: {raw_msg.type_}")
            return []

    def _handle_book_update(self, raw_msg: RawWsMessage) -> list[WsEvent]:
        """Handle book update message."""
        try:
            data = BookUpdateData.from_dict(raw_msg.data)
        except Exception as e:
            logger.warning(f"Failed to parse book update: {e}")
            return [WsEvent.error(MessageParseError(str(e)))]

        # Check for resync signal
        if data.resync:
            logger.info(f"Resync required for orderbook: {data.orderbook_id}")
            return [WsEvent.resync_required(data.orderbook_id)]

        orderbook_id = data.orderbook_id
        is_snapshot = data.is_snapshot

        # Get or create orderbook state
        if orderbook_id not in self._orderbooks:
            self._orderbooks[orderbook_id] = LocalOrderbook(orderbook_id)

        book = self._orderbooks[orderbook_id]

        try:
            book.apply_update(data)
            return [WsEvent.book_update(orderbook_id, is_snapshot)]
        except SequenceGapError as e:
            logger.warning(
                f"Sequence gap in orderbook {orderbook_id}: "
                f"expected {e.expected}, received {e.received}"
            )
            book.clear()
            return [WsEvent.resync_required(orderbook_id)]
        except Exception as e:
            return [WsEvent.error(e)]

    def _handle_trade(self, raw_msg: RawWsMessage) -> list[WsEvent]:
        """Handle trade message."""
        try:
            data = TradeData.from_dict(raw_msg.data)
        except Exception as e:
            logger.warning(f"Failed to parse trade: {e}")
            return [WsEvent.error(MessageParseError(str(e)))]

        return [WsEvent.trade(data.orderbook_id, data)]

    def _handle_user_event(self, raw_msg: RawWsMessage) -> list[WsEvent]:
        """Handle user event message."""
        try:
            data = UserEventData.from_dict(raw_msg.data)
        except Exception as e:
            logger.warning(f"Failed to parse user event: {e}")
            return [WsEvent.error(MessageParseError(str(e)))]

        event_type = data.event_type

        # Use the tracked subscribed user (single user per connection)
        user = self._subscribed_user or "unknown"

        # Update local state for the subscribed user
        if user in self._user_states:
            self._user_states[user].apply_event(data)

        return [WsEvent.user_update(event_type, user)]

    def _handle_price_history(self, raw_msg: RawWsMessage) -> list[WsEvent]:
        """Handle price history message."""
        try:
            data = PriceHistoryData.from_dict(raw_msg.data)
        except Exception as e:
            logger.warning(f"Failed to parse price history: {e}")
            return [WsEvent.error(MessageParseError(str(e)))]

        # Heartbeats don't have orderbook_id
        if data.event_type == "heartbeat":
            for history in self._price_histories.values():
                history.apply_heartbeat(data)
            return []

        orderbook_id = data.orderbook_id
        if not orderbook_id:
            logger.warning("Price history message missing orderbook_id")
            return []

        resolution = data.resolution or "1m"

        # Update local state
        key = PriceHistoryKey(orderbook_id, resolution)

        if key in self._price_histories:
            self._price_histories[key].apply_event(data)
        elif data.event_type == "snapshot":
            # Create new history if this is a snapshot
            history = PriceHistory(
                orderbook_id,
                resolution,
                data.include_ohlcv or False,
            )
            history.apply_event(data)
            self._price_histories[key] = history

        return [WsEvent.price_update(orderbook_id, resolution)]

    def _handle_market_event(self, raw_msg: RawWsMessage) -> list[WsEvent]:
        """Handle market event message."""
        try:
            data = MarketEventData.from_dict(raw_msg.data)
        except Exception as e:
            logger.warning(f"Failed to parse market event: {e}")
            return [WsEvent.error(MessageParseError(str(e)))]

        return [WsEvent.market_event(data.event_type, data.market_pubkey)]

    def _handle_error(self, raw_msg: RawWsMessage) -> list[WsEvent]:
        """Handle error message from server."""
        try:
            data = ErrorData.from_dict(raw_msg.data)
        except Exception as e:
            logger.warning(f"Failed to parse error: {e}")
            return [WsEvent.error(MessageParseError(str(e)))]

        logger.error(f"Server error: {data.error} (code: {data.code})")

        from .error import ServerError

        return [WsEvent.error(ServerError(data.code, data.error))]

    def init_orderbook(self, orderbook_id: str) -> None:
        """Initialize orderbook state for a subscription."""
        if orderbook_id not in self._orderbooks:
            self._orderbooks[orderbook_id] = LocalOrderbook(orderbook_id)

    def init_user_state(self, user: str) -> None:
        """Initialize user state for a subscription."""
        self._subscribed_user = user
        if user not in self._user_states:
            self._user_states[user] = UserState(user)

    def clear_subscribed_user(self, user: str) -> None:
        """Clear the subscribed user."""
        if self._subscribed_user == user:
            self._subscribed_user = None

    def init_price_history(
        self,
        orderbook_id: str,
        resolution: str,
        include_ohlcv: bool,
    ) -> None:
        """Initialize price history state for a subscription."""
        key = PriceHistoryKey(orderbook_id, resolution)
        if key not in self._price_histories:
            self._price_histories[key] = PriceHistory(
                orderbook_id,
                resolution,
                include_ohlcv,
            )

    def get_orderbook(self, orderbook_id: str) -> Optional[LocalOrderbook]:
        """Get orderbook state."""
        return self._orderbooks.get(orderbook_id)

    def get_user_state(self, user: str) -> Optional[UserState]:
        """Get user state."""
        return self._user_states.get(user)

    def get_price_history(
        self,
        orderbook_id: str,
        resolution: str,
    ) -> Optional[PriceHistory]:
        """Get price history state."""
        key = PriceHistoryKey(orderbook_id, resolution)
        return self._price_histories.get(key)

    def clear_orderbook(self, orderbook_id: str) -> None:
        """Clear orderbook state."""
        if orderbook_id in self._orderbooks:
            self._orderbooks[orderbook_id].clear()

    def clear_user_state(self, user: str) -> None:
        """Clear user state."""
        if user in self._user_states:
            self._user_states[user].clear()

    def clear_price_history(self, orderbook_id: str, resolution: str) -> None:
        """Clear price history state."""
        key = PriceHistoryKey(orderbook_id, resolution)
        if key in self._price_histories:
            self._price_histories[key].clear()

    def clear_all(self) -> None:
        """Clear all state."""
        self._orderbooks.clear()
        self._user_states.clear()
        self._price_histories.clear()
