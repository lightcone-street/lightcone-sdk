"""WebSocket client module for Lightcone.

This module provides real-time data streaming functionality for
orderbook updates, trades, user events, and more.

Example:
    ```python
    from lightcone_sdk.websocket import LightconeWebSocketClient, WsEventType

    client = await LightconeWebSocketClient.connect("wss://ws.lightcone.xyz")

    await client.subscribe_book_updates(["market:ob1"])

    async for event in client:
        if event.type == WsEventType.BOOK_UPDATE:
            book = client.get_orderbook(event.orderbook_id)
            print(f"Best bid: {book.best_bid()}")

    await client.disconnect()
    ```
"""

from .client import LightconeWebSocketClient

from .error import (
    WebSocketError,
    ConnectionFailedError,
    ConnectionClosedError,
    RateLimitedError,
    MessageParseError,
    SequenceGapError,
    ResyncRequiredError,
    SubscriptionFailedError,
    PingTimeoutError,
    ProtocolError,
    ServerError,
    NotConnectedError,
    AlreadyConnectedError,
    SendFailedError,
    ChannelClosedError,
    InvalidUrlError,
    OperationTimeoutError,
    AuthenticationFailedError,
    AuthRequiredError,
)

from .types import (
    # Request types
    WsRequest,
    SubscribeType,
    book_update_params,
    trades_params,
    user_params,
    price_history_params,
    market_params,
    # Response types
    RawWsMessage,
    PriceLevel,
    BookUpdateData,
    TradeData,
    OutcomeBalance,
    Balance,
    BalanceEntry,
    Order,
    OrderUpdate,
    UserEventData,
    Candle,
    PriceHistoryData,
    MarketEventData,
    MarketEventType,
    ErrorData,
    ErrorCode,
    # Event types
    WsEventType,
    WsEvent,
    MessageType,
    Side,
    PriceLevelSide,
)

from .subscriptions import (
    Subscription,
    SubscriptionManager,
)

from .state import (
    LocalOrderbook,
    PriceHistory,
    PriceHistoryKey,
    UserState,
)

from .handlers import MessageHandler

from ..auth import (
    AUTH_API_URL,
    AuthCredentials,
    authenticate,
    generate_signin_message,
    generate_signin_message_with_timestamp,
    sign_message,
)

__all__ = [
    # Client
    "LightconeWebSocketClient",
    # Errors
    "WebSocketError",
    "ConnectionFailedError",
    "ConnectionClosedError",
    "RateLimitedError",
    "MessageParseError",
    "SequenceGapError",
    "ResyncRequiredError",
    "SubscriptionFailedError",
    "PingTimeoutError",
    "ProtocolError",
    "ServerError",
    "NotConnectedError",
    "AlreadyConnectedError",
    "SendFailedError",
    "ChannelClosedError",
    "InvalidUrlError",
    "OperationTimeoutError",
    "AuthenticationFailedError",
    "AuthRequiredError",
    # Request types
    "WsRequest",
    "SubscribeType",
    "book_update_params",
    "trades_params",
    "user_params",
    "price_history_params",
    "market_params",
    # Response types
    "RawWsMessage",
    "PriceLevel",
    "BookUpdateData",
    "TradeData",
    "OutcomeBalance",
    "Balance",
    "BalanceEntry",
    "Order",
    "OrderUpdate",
    "UserEventData",
    "Candle",
    "PriceHistoryData",
    "MarketEventData",
    "MarketEventType",
    "ErrorData",
    "ErrorCode",
    # Event types
    "WsEventType",
    "WsEvent",
    "MessageType",
    "Side",
    "PriceLevelSide",
    # Subscriptions
    "Subscription",
    "SubscriptionManager",
    # State
    "LocalOrderbook",
    "PriceHistory",
    "PriceHistoryKey",
    "UserState",
    # Handlers
    "MessageHandler",
    # Authentication
    "AUTH_API_URL",
    "AuthCredentials",
    "authenticate",
    "generate_signin_message",
    "generate_signin_message_with_timestamp",
    "sign_message",
]
