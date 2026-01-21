"""Lightcone WebSocket client implementation."""

import asyncio
import json
import logging
from typing import Optional, AsyncIterator, Union

import websockets
from websockets.client import WebSocketClientProtocol
from websockets.exceptions import ConnectionClosed, InvalidURI

from .error import (
    WebSocketError,
    ConnectionFailedError,
    ConnectionClosedError,
    NotConnectedError,
    AlreadyConnectedError,
    SendFailedError,
    InvalidUrlError,
    RateLimitedError,
)
from .handlers import MessageHandler
from .subscriptions import SubscriptionManager
from .state import LocalOrderbook, PriceHistory, UserState
from .types import WsEvent, WsRequest

logger = logging.getLogger(__name__)


class LightconeWebSocketClient:
    """Lightcone WebSocket client for real-time data streaming.

    Provides methods for subscribing to orderbook updates, trades, user events,
    price history, and market events.

    Example:
        ```python
        client = await LightconeWebSocketClient.connect("wss://ws.lightcone.xyz")

        await client.subscribe_book_updates(["market:ob1"])

        async for event in client:
            if event.type == WsEventType.BOOK_UPDATE:
                book = client.get_orderbook(event.orderbook_id)
                print(f"Best bid: {book.best_bid()}")

        await client.disconnect()
        ```
    """

    def __init__(
        self,
        url: str,
        reconnect: bool = True,
        max_reconnect_attempts: int = 5,
        reconnect_delay: float = 1.0,
    ):
        """Create a new WebSocket client.

        Args:
            url: WebSocket URL to connect to
            reconnect: Whether to automatically reconnect on disconnect
            max_reconnect_attempts: Maximum number of reconnect attempts
            reconnect_delay: Initial delay between reconnect attempts (exponential backoff)
        """
        self._url = url
        self._reconnect = reconnect
        self._max_reconnect_attempts = max_reconnect_attempts
        self._reconnect_delay = reconnect_delay

        self._ws: Optional[WebSocketClientProtocol] = None
        self._handler = MessageHandler()
        self._subscriptions = SubscriptionManager()
        self._connected = False
        self._running = False
        self._event_queue: asyncio.Queue[WsEvent] = asyncio.Queue()
        self._receive_task: Optional[asyncio.Task] = None

    @classmethod
    async def connect(
        cls,
        url: str,
        reconnect: bool = True,
        max_reconnect_attempts: int = 5,
        reconnect_delay: float = 1.0,
    ) -> "LightconeWebSocketClient":
        """Connect to the WebSocket server.

        Args:
            url: WebSocket URL to connect to
            reconnect: Whether to automatically reconnect on disconnect
            max_reconnect_attempts: Maximum number of reconnect attempts
            reconnect_delay: Initial delay between reconnect attempts

        Returns:
            Connected LightconeWebSocketClient instance

        Raises:
            ConnectionFailedError: If connection fails
            InvalidUrlError: If URL is invalid
        """
        client = cls(url, reconnect, max_reconnect_attempts, reconnect_delay)
        await client._connect()
        return client

    async def _connect(self) -> None:
        """Internal connect implementation."""
        if self._connected:
            raise AlreadyConnectedError()

        try:
            self._ws = await websockets.connect(
                self._url,
                ping_interval=30,
                ping_timeout=10,
                close_timeout=5,
            )
            self._connected = True
            self._running = True

            # Start the receive loop
            self._receive_task = asyncio.create_task(self._receive_loop())

            # Queue connected event
            await self._event_queue.put(WsEvent.connected())

            logger.info(f"Connected to WebSocket: {self._url}")

        except InvalidURI as e:
            raise InvalidUrlError(self._url) from e
        except Exception as e:
            raise ConnectionFailedError(str(e)) from e

    async def disconnect(self) -> None:
        """Disconnect from the WebSocket server."""
        self._running = False
        self._connected = False

        if self._receive_task:
            self._receive_task.cancel()
            try:
                await self._receive_task
            except asyncio.CancelledError:
                pass
            self._receive_task = None

        if self._ws:
            try:
                await self._ws.close()
            except Exception:
                pass
            self._ws = None

        # Clear state
        self._handler.clear_all()
        self._subscriptions.clear()

        logger.info("Disconnected from WebSocket")

    async def _receive_loop(self) -> None:
        """Background task that receives messages from the WebSocket."""
        reconnect_attempts = 0

        while self._running:
            try:
                if not self._ws:
                    break

                message = await self._ws.recv()

                if isinstance(message, str):
                    events = await self._handler.handle_message(message)
                    for event in events:
                        await self._event_queue.put(event)

            except ConnectionClosed as e:
                self._connected = False

                # Check if rate limited
                if e.code == 1008:
                    logger.warning("Rate limited by server")
                    await self._event_queue.put(WsEvent.error(RateLimitedError()))
                    break

                reason = e.reason or "Connection closed"
                logger.warning(f"WebSocket connection closed: {reason}")

                await self._event_queue.put(WsEvent.disconnected(reason))

                # Attempt reconnect if enabled
                if self._reconnect and self._running:
                    await self._attempt_reconnect(reconnect_attempts)
                    reconnect_attempts += 1
                else:
                    break

            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Error in receive loop: {e}")
                await self._event_queue.put(WsEvent.error(WebSocketError(str(e))))

    async def _attempt_reconnect(self, attempt: int) -> None:
        """Attempt to reconnect to the server."""
        if attempt >= self._max_reconnect_attempts:
            logger.error("Max reconnect attempts reached")
            self._running = False
            return

        delay = self._reconnect_delay * (2**attempt)
        logger.info(f"Reconnecting in {delay}s (attempt {attempt + 1})")

        await self._event_queue.put(WsEvent.reconnecting(attempt + 1))
        await asyncio.sleep(delay)

        try:
            self._ws = await websockets.connect(
                self._url,
                ping_interval=30,
                ping_timeout=10,
                close_timeout=5,
            )
            self._connected = True

            # Re-subscribe to all channels
            await self._resubscribe()

            await self._event_queue.put(WsEvent.connected())
            logger.info("Reconnected successfully")

        except Exception as e:
            logger.warning(f"Reconnect failed: {e}")

    async def _resubscribe(self) -> None:
        """Re-subscribe to all channels after reconnect."""
        for sub in self._subscriptions.get_all_subscriptions():
            try:
                await self._send_subscribe(sub.to_params())
            except Exception as e:
                logger.warning(f"Failed to resubscribe: {e}")

    async def _send(self, request: WsRequest) -> None:
        """Send a request to the server."""
        if not self._ws or not self._connected:
            raise NotConnectedError()

        try:
            message = json.dumps(request.to_dict())
            await self._ws.send(message)
        except Exception as e:
            raise SendFailedError(str(e)) from e

    async def _send_subscribe(self, params: dict) -> None:
        """Send a subscribe request."""
        await self._send(WsRequest.subscribe(params))

    async def _send_unsubscribe(self, params: dict) -> None:
        """Send an unsubscribe request."""
        await self._send(WsRequest.unsubscribe(params))

    # =========================================================================
    # Subscribe methods
    # =========================================================================

    async def subscribe_book_updates(self, orderbook_ids: list[str]) -> None:
        """Subscribe to orderbook updates.

        Args:
            orderbook_ids: List of orderbook identifiers
        """
        from .types import book_update_params

        # Initialize state
        for ob_id in orderbook_ids:
            self._handler.init_orderbook(ob_id)

        # Track subscription
        self._subscriptions.add_book_update(orderbook_ids)

        # Send subscribe
        await self._send_subscribe(book_update_params(orderbook_ids))

    async def subscribe_trades(self, orderbook_ids: list[str]) -> None:
        """Subscribe to trade executions.

        Args:
            orderbook_ids: List of orderbook identifiers
        """
        from .types import trades_params

        self._subscriptions.add_trades(orderbook_ids)
        await self._send_subscribe(trades_params(orderbook_ids))

    async def subscribe_user(self, user: str) -> None:
        """Subscribe to user events (orders and balances).

        Args:
            user: User's public key
        """
        from .types import user_params

        self._handler.init_user_state(user)
        self._subscriptions.add_user(user)
        await self._send_subscribe(user_params(user))

    async def subscribe_price_history(
        self,
        orderbook_id: str,
        resolution: str,
        include_ohlcv: bool = False,
    ) -> None:
        """Subscribe to price history updates.

        Args:
            orderbook_id: Orderbook identifier
            resolution: Candle resolution (1m, 5m, 15m, 1h, 4h, 1d)
            include_ohlcv: Include OHLCV data in addition to midpoint
        """
        from .types import price_history_params

        self._handler.init_price_history(orderbook_id, resolution, include_ohlcv)
        self._subscriptions.add_price_history(orderbook_id, resolution, include_ohlcv)
        await self._send_subscribe(
            price_history_params(orderbook_id, resolution, include_ohlcv)
        )

    async def subscribe_market(self, market_pubkey: str) -> None:
        """Subscribe to market events.

        Args:
            market_pubkey: Market's public key
        """
        from .types import market_params

        self._subscriptions.add_market(market_pubkey)
        await self._send_subscribe(market_params(market_pubkey))

    # =========================================================================
    # Unsubscribe methods
    # =========================================================================

    async def unsubscribe_book_updates(self, orderbook_ids: list[str]) -> None:
        """Unsubscribe from orderbook updates.

        Args:
            orderbook_ids: List of orderbook identifiers
        """
        from .types import book_update_params

        self._subscriptions.remove_book_update(orderbook_ids)
        await self._send_unsubscribe(book_update_params(orderbook_ids))

    async def unsubscribe_trades(self, orderbook_ids: list[str]) -> None:
        """Unsubscribe from trade executions.

        Args:
            orderbook_ids: List of orderbook identifiers
        """
        from .types import trades_params

        self._subscriptions.remove_trades(orderbook_ids)
        await self._send_unsubscribe(trades_params(orderbook_ids))

    async def unsubscribe_user(self, user: str) -> None:
        """Unsubscribe from user events.

        Args:
            user: User's public key
        """
        from .types import user_params

        self._subscriptions.remove_user(user)
        await self._send_unsubscribe(user_params(user))

    async def unsubscribe_price_history(
        self,
        orderbook_id: str,
        resolution: str,
    ) -> None:
        """Unsubscribe from price history updates.

        Args:
            orderbook_id: Orderbook identifier
            resolution: Candle resolution
        """
        from .types import price_history_params

        self._subscriptions.remove_price_history(orderbook_id, resolution)
        await self._send_unsubscribe(
            price_history_params(orderbook_id, resolution, False)
        )

    async def unsubscribe_market(self, market_pubkey: str) -> None:
        """Unsubscribe from market events.

        Args:
            market_pubkey: Market's public key
        """
        from .types import market_params

        self._subscriptions.remove_market(market_pubkey)
        await self._send_unsubscribe(market_params(market_pubkey))

    # =========================================================================
    # State access methods
    # =========================================================================

    def get_orderbook(self, orderbook_id: str) -> Optional[LocalOrderbook]:
        """Get local orderbook state.

        Args:
            orderbook_id: Orderbook identifier

        Returns:
            LocalOrderbook if subscribed, None otherwise
        """
        return self._handler.get_orderbook(orderbook_id)

    def get_user_state(self, user: str) -> Optional[UserState]:
        """Get local user state.

        Args:
            user: User's public key

        Returns:
            UserState if subscribed, None otherwise
        """
        return self._handler.get_user_state(user)

    def get_price_history(
        self,
        orderbook_id: str,
        resolution: str,
    ) -> Optional[PriceHistory]:
        """Get local price history state.

        Args:
            orderbook_id: Orderbook identifier
            resolution: Candle resolution

        Returns:
            PriceHistory if subscribed, None otherwise
        """
        return self._handler.get_price_history(orderbook_id, resolution)

    # =========================================================================
    # Utility methods
    # =========================================================================

    async def ping(self) -> None:
        """Send a ping to the server."""
        await self._send(WsRequest.ping())

    @property
    def is_connected(self) -> bool:
        """Check if connected to the server."""
        return self._connected

    # =========================================================================
    # Async iteration
    # =========================================================================

    def __aiter__(self) -> AsyncIterator[WsEvent]:
        """Return async iterator for events."""
        return self

    async def __anext__(self) -> WsEvent:
        """Get the next event."""
        if not self._running and self._event_queue.empty():
            raise StopAsyncIteration

        try:
            event = await self._event_queue.get()
            return event
        except asyncio.CancelledError:
            raise StopAsyncIteration

    async def recv(self) -> WsEvent:
        """Receive the next event.

        This is an alternative to async iteration for getting events.

        Returns:
            The next WsEvent

        Raises:
            WebSocketError: If not connected or receive fails
        """
        if not self._running:
            raise NotConnectedError()

        return await self._event_queue.get()
