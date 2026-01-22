"""Lightcone WebSocket client implementation."""

import asyncio
import json
import logging
import random
import time
from dataclasses import dataclass
from typing import Optional, AsyncIterator, Union
from urllib.parse import quote

from websockets.asyncio.client import connect as ws_connect, ClientConnection
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
    PingTimeoutError,
)
from .handlers import MessageHandler
from .subscriptions import SubscriptionManager
from .state import LocalOrderbook, PriceHistory, UserState
from .types import WsEvent, WsRequest

logger = logging.getLogger(__name__)

# Connection timeout in seconds
CONNECTION_TIMEOUT_SECS = 30


@dataclass
class WebSocketConfig:
    """Configuration for WebSocket connection."""

    ping_interval_secs: float = 30.0
    ping_timeout_secs: float = 10.0
    pong_timeout_secs: float = 60.0
    close_timeout_secs: float = 5.0
    max_reconnect_attempts: int = 5
    reconnect_delay: float = 1.0
    max_delay: float = 30.0
    event_queue_size: int = 10000


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
        max_delay: float = 30.0,
        auth_token: Optional[str] = None,
        config: Optional[WebSocketConfig] = None,
    ):
        """Create a new WebSocket client.

        Args:
            url: WebSocket URL to connect to
            reconnect: Whether to automatically reconnect on disconnect
            max_reconnect_attempts: Maximum number of reconnect attempts
            reconnect_delay: Initial delay between reconnect attempts (exponential backoff)
            max_delay: Maximum delay between reconnect attempts in seconds
            auth_token: Optional authentication token for private streams
            config: Optional WebSocket configuration
        """
        self._url = url
        self._reconnect = reconnect
        self._config = config or WebSocketConfig(
            max_reconnect_attempts=max_reconnect_attempts,
            reconnect_delay=reconnect_delay,
            max_delay=max_delay,
        )
        self._auth_token = auth_token

        self._ws: Optional[ClientConnection] = None
        self._handler = MessageHandler()
        self._subscriptions = SubscriptionManager()
        self._connected = False
        self._running = False
        self._event_queue: asyncio.Queue[WsEvent] = asyncio.Queue(
            maxsize=self._config.event_queue_size
        )
        self._receive_task: Optional[asyncio.Task] = None
        self._state_lock = asyncio.Lock()

        # Pong timeout tracking
        self._last_pong: float = time.time()
        self._awaiting_pong: bool = False

    @classmethod
    async def connect(
        cls,
        url: str,
        reconnect: bool = True,
        max_reconnect_attempts: int = 5,
        reconnect_delay: float = 1.0,
        max_delay: float = 30.0,
        auth_token: Optional[str] = None,
        config: Optional[WebSocketConfig] = None,
    ) -> "LightconeWebSocketClient":
        """Connect to the WebSocket server.

        Args:
            url: WebSocket URL to connect to
            reconnect: Whether to automatically reconnect on disconnect
            max_reconnect_attempts: Maximum number of reconnect attempts
            reconnect_delay: Initial delay between reconnect attempts
            max_delay: Maximum delay between reconnect attempts in seconds
            auth_token: Optional authentication token for private streams
            config: Optional WebSocket configuration

        Returns:
            Connected LightconeWebSocketClient instance

        Raises:
            ConnectionFailedError: If connection fails
            InvalidUrlError: If URL is invalid
        """
        client = cls(
            url,
            reconnect,
            max_reconnect_attempts,
            reconnect_delay,
            max_delay,
            auth_token,
            config,
        )
        await client._connect()
        return client

    @classmethod
    async def connect_authenticated(
        cls,
        signing_key,  # SigningKey from nacl.signing
        url: str = "wss://ws.lightcone.xyz/ws",
        reconnect: bool = True,
        max_reconnect_attempts: int = 5,
        reconnect_delay: float = 1.0,
        max_delay: float = 30.0,
    ) -> "LightconeWebSocketClient":
        """Connect to the WebSocket server with authentication.

        This method authenticates with the Lightcone API and then connects
        to the WebSocket server with the obtained auth token.

        Args:
            signing_key: The Ed25519 signing key for authentication
            url: WebSocket URL to connect to
            reconnect: Whether to automatically reconnect on disconnect
            max_reconnect_attempts: Maximum number of reconnect attempts
            reconnect_delay: Initial delay between reconnect attempts
            max_delay: Maximum delay between reconnect attempts in seconds

        Returns:
            Connected and authenticated LightconeWebSocketClient instance

        Raises:
            WebSocketError: If authentication fails
            ConnectionFailedError: If connection fails
        """
        from .auth import authenticate

        credentials = await authenticate(signing_key)
        return await cls.connect(
            url,
            reconnect,
            max_reconnect_attempts,
            reconnect_delay,
            max_delay,
            credentials.auth_token,
        )

    async def _connect(self) -> None:
        """Internal connect implementation."""
        async with self._state_lock:
            if self._connected:
                raise AlreadyConnectedError()

        try:
            # Build extra headers for authentication with proper encoding
            extra_headers = {}
            if self._auth_token:
                extra_headers["Cookie"] = f"auth_token={quote(self._auth_token, safe='')}"

            try:
                self._ws = await asyncio.wait_for(
                    ws_connect(
                        self._url,
                        ping_interval=self._config.ping_interval_secs,
                        ping_timeout=self._config.ping_timeout_secs,
                        close_timeout=self._config.close_timeout_secs,
                        additional_headers=extra_headers if extra_headers else None,
                    ),
                    timeout=CONNECTION_TIMEOUT_SECS,
                )
            except asyncio.TimeoutError:
                raise ConnectionFailedError("Connection timed out")
            async with self._state_lock:
                self._connected = True
                self._running = True
                self._last_pong = time.time()
                self._awaiting_pong = False

            # Start the receive loop
            self._receive_task = asyncio.create_task(self._receive_loop())

            # Queue connected event
            await self._queue_event(WsEvent.connected())

            logger.info(f"Connected to WebSocket: {self._url}")

        except InvalidURI as e:
            raise InvalidUrlError(self._url) from e
        except Exception as e:
            raise ConnectionFailedError(str(e)) from e

    async def disconnect(self) -> None:
        """Disconnect from the WebSocket server."""
        async with self._state_lock:
            self._running = False
            self._connected = False

        # Close socket first to unblock recv() in receive loop
        if self._ws:
            try:
                await self._ws.close()
            except Exception:
                pass
            self._ws = None

        # Then cancel the receive task
        if self._receive_task:
            self._receive_task.cancel()
            try:
                await self._receive_task
            except asyncio.CancelledError:
                pass
            self._receive_task = None

        # Clear state
        self._handler.clear_all()
        self._subscriptions.clear()

        logger.info("Disconnected from WebSocket")

    async def _queue_event(self, event: WsEvent) -> None:
        """Queue an event, dropping oldest if queue is full."""
        try:
            self._event_queue.put_nowait(event)
        except asyncio.QueueFull:
            logger.warning("Event queue full, dropping oldest event")
            try:
                self._event_queue.get_nowait()  # Drop oldest
            except asyncio.QueueEmpty:
                pass
            try:
                self._event_queue.put_nowait(event)
            except asyncio.QueueFull:
                logger.error("Failed to queue event after dropping oldest")

    async def _receive_loop(self) -> None:
        """Background task that receives messages from the WebSocket."""
        reconnect_attempts = 0

        while self._running:
            try:
                if not self._ws:
                    break

                # Check for pong timeout
                if self._awaiting_pong:
                    elapsed = time.time() - self._last_pong
                    if elapsed > self._config.pong_timeout_secs:
                        logger.warning(f"Pong timeout after {elapsed:.1f}s")
                        await self._queue_event(WsEvent.error(PingTimeoutError()))
                        async with self._state_lock:
                            self._connected = False
                        if self._reconnect and self._running:
                            await self._attempt_reconnect(reconnect_attempts)
                            reconnect_attempts += 1
                            continue
                        else:
                            break

                message = await self._ws.recv()

                if isinstance(message, str):
                    events = await self._handler.handle_message(message)
                    for event in events:
                        # Track pong responses
                        if event.type.name == "PONG":
                            self._last_pong = time.time()
                            self._awaiting_pong = False
                        await self._queue_event(event)

            except ConnectionClosed as e:
                async with self._state_lock:
                    self._connected = False

                # Check if rate limited
                if e.code == 1008:
                    logger.warning("Rate limited by server")
                    await self._queue_event(WsEvent.error(RateLimitedError()))
                    break

                reason = e.reason or "Connection closed"
                logger.warning(f"WebSocket connection closed: {reason}")

                await self._queue_event(WsEvent.disconnected(reason))

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
                await self._queue_event(WsEvent.error(WebSocketError(str(e))))

    async def _attempt_reconnect(self, attempt: int) -> None:
        """Attempt to reconnect to the server."""
        if attempt >= self._config.max_reconnect_attempts:
            logger.error("Max reconnect attempts reached")
            async with self._state_lock:
                self._running = False
            return

        # Full jitter: randomize between 0 and exponential delay to prevent thundering herd
        max_delay = self._config.reconnect_delay * (2**attempt)
        jittered_delay = random.uniform(0, max_delay)
        delay = min(jittered_delay, self._config.max_delay)
        logger.info(f"Reconnecting in {delay:.2f}s (attempt {attempt + 1})")

        await self._queue_event(WsEvent.reconnecting(attempt + 1))
        await asyncio.sleep(delay)

        try:
            # Build extra headers for authentication with proper encoding
            extra_headers = {}
            if self._auth_token:
                extra_headers["Cookie"] = f"auth_token={quote(self._auth_token, safe='')}"

            try:
                self._ws = await asyncio.wait_for(
                    ws_connect(
                        self._url,
                        ping_interval=self._config.ping_interval_secs,
                        ping_timeout=self._config.ping_timeout_secs,
                        close_timeout=self._config.close_timeout_secs,
                        additional_headers=extra_headers if extra_headers else None,
                    ),
                    timeout=CONNECTION_TIMEOUT_SECS,
                )
            except asyncio.TimeoutError:
                logger.warning("Reconnect timed out")
                return
            async with self._state_lock:
                self._connected = True
                self._last_pong = time.time()
                self._awaiting_pong = False

            # Re-subscribe to all channels
            await self._resubscribe()

            await self._queue_event(WsEvent.connected())
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

        Raises:
            ValueError: If orderbook_ids is empty or contains invalid IDs
        """
        if not orderbook_ids:
            raise ValueError("orderbook_ids cannot be empty")
        for ob_id in orderbook_ids:
            if not ob_id or not ob_id.strip():
                raise ValueError(f"Invalid orderbook_id: empty string")
            if ":" not in ob_id:
                raise ValueError(f"Invalid orderbook_id format: {ob_id} (expected 'market:id')")

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

        Raises:
            ValueError: If orderbook_ids is empty or contains invalid IDs
        """
        if not orderbook_ids:
            raise ValueError("orderbook_ids cannot be empty")
        for ob_id in orderbook_ids:
            if not ob_id or not ob_id.strip():
                raise ValueError(f"Invalid orderbook_id: empty string")

        from .types import trades_params

        self._subscriptions.add_trades(orderbook_ids)
        await self._send_subscribe(trades_params(orderbook_ids))

    async def subscribe_user(self, user: str) -> None:
        """Subscribe to user events (orders and balances).

        Args:
            user: User's public key

        Raises:
            ValueError: If user is empty
        """
        if not user or not user.strip():
            raise ValueError("user cannot be empty")

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

        Raises:
            ValueError: If orderbook_id or resolution is empty/invalid
        """
        if not orderbook_id or not orderbook_id.strip():
            raise ValueError("orderbook_id cannot be empty")
        valid_resolutions = {"1m", "5m", "15m", "1h", "4h", "1d"}
        if resolution not in valid_resolutions:
            raise ValueError(f"Invalid resolution: {resolution}. Must be one of {valid_resolutions}")

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

        Raises:
            ValueError: If market_pubkey is empty
        """
        if not market_pubkey or not market_pubkey.strip():
            raise ValueError("market_pubkey cannot be empty")

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

        self._handler.clear_subscribed_user(user)
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
        self._awaiting_pong = True
        await self._send(WsRequest.ping())

    @property
    def is_connected(self) -> bool:
        """Check if connected to the server."""
        return self._connected

    def is_task_running(self) -> bool:
        """Check if the receive task is still running.

        Returns:
            True if the task exists and is not done, False otherwise.
        """
        return self._receive_task is not None and not self._receive_task.done()

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
