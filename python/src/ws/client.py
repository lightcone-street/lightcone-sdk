"""WebSocket client for the Lightcone SDK.

Matches TS ws/client.node.ts with reconnection, ping/pong, and event callbacks.
"""

import asyncio
import json
import logging
from typing import Any, Callable, Optional

import aiohttp

from ..error import WsError
from . import (
    MessageIn,
    ReadyState,
    WsConfig,
    WsEvent,
    WsEventType,
    WS_DEFAULT_CONFIG,
    parse_message_in,
    ping as make_ping,
)
from .subscriptions import SubscribeParams, subscription_key

logger = logging.getLogger(__name__)


class WsClient:
    """Async WebSocket client with reconnection and subscription tracking."""

    def __init__(self, config: Optional[WsConfig] = None):
        self._config = config or WS_DEFAULT_CONFIG
        self._session: Optional[aiohttp.ClientSession] = None
        self._ws: Optional[aiohttp.ClientWebSocketResponse] = None
        self._state = ReadyState.CLOSED
        self._callbacks: list[Callable[[WsEvent], Any]] = []
        self._subscriptions: dict[str, dict] = {}
        self._reconnect_attempts = 0
        self._ping_task: Optional[asyncio.Task] = None
        self._receive_task: Optional[asyncio.Task] = None
        self._should_reconnect = False
        self._auth_token: Optional[str] = None
        self._pending_messages: list[dict] = []

    def ready_state(self) -> ReadyState:
        return self._state

    def is_connected(self) -> bool:
        return self._state == ReadyState.OPEN

    def set_auth_token(self, token: Optional[str]) -> None:
        self._auth_token = token

    def on(self, callback: Callable[[WsEvent], Any]) -> Callable[[], None]:
        """Register an event callback. Returns an unsubscribe function."""
        self._callbacks.append(callback)

        def unsubscribe():
            if callback in self._callbacks:
                self._callbacks.remove(callback)

        return unsubscribe

    def _emit(self, event: WsEvent) -> None:
        for cb in self._callbacks:
            try:
                result = cb(event)
                if asyncio.iscoroutine(result):
                    asyncio.ensure_future(result)
            except Exception as e:
                logger.warning(f"Callback error: {e}")

    async def connect(self) -> None:
        """Connect to the WebSocket server."""
        if self._state == ReadyState.OPEN:
            return

        self._state = ReadyState.CONNECTING
        self._should_reconnect = self._config.reconnect

        try:
            self._session = aiohttp.ClientSession()
            headers: dict[str, str] = {}
            if self._auth_token:
                headers["Cookie"] = f"auth_token={self._auth_token}"

            self._ws = await self._session.ws_connect(
                self._config.url,
                headers=headers,
            )
            self._state = ReadyState.OPEN
            self._reconnect_attempts = 0

            self._emit(WsEvent(type=WsEventType.CONNECTED))

            # Re-subscribe to previous subscriptions
            for sub_msg in self._subscriptions.values():
                await self._send_raw(sub_msg)

            # Send pending messages
            for msg in self._pending_messages:
                await self._send_raw(msg)
            self._pending_messages.clear()

            # Start receive loop and ping
            self._receive_task = asyncio.ensure_future(self._receive_loop())
            self._ping_task = asyncio.ensure_future(self._ping_loop())

        except Exception as e:
            self._state = ReadyState.CLOSED
            self._emit(WsEvent(type=WsEventType.ERROR, error=str(e)))
            if self._should_reconnect:
                await self._try_reconnect()
            else:
                raise WsError.connection_failed(str(e))

    async def disconnect(self) -> None:
        """Disconnect from the WebSocket server."""
        self._should_reconnect = False
        self._state = ReadyState.CLOSING

        if self._ping_task:
            self._ping_task.cancel()
            self._ping_task = None

        if self._receive_task:
            self._receive_task.cancel()
            self._receive_task = None

        if self._ws and not self._ws.closed:
            await self._ws.close()

        if self._session and not self._session.closed:
            await self._session.close()

        self._ws = None
        self._session = None
        self._state = ReadyState.CLOSED
        self._emit(WsEvent(type=WsEventType.DISCONNECTED))

    async def send(self, message: dict) -> None:
        """Send a message. Queues if not connected."""
        if self._state == ReadyState.OPEN:
            await self._send_raw(message)
        else:
            self._pending_messages.append(message)

    async def subscribe(self, params: dict) -> None:
        """Subscribe to a channel. Tracks subscription for reconnection."""
        message = {"method": "subscribe", "params": params}
        key = f"{params.get('type', '')}:{json.dumps(params, sort_keys=True)}"
        self._subscriptions[key] = message
        await self.send(message)

    async def unsubscribe(self, params: dict) -> None:
        """Unsubscribe from a channel."""
        message = {"method": "unsubscribe", "params": params}
        key = f"{params.get('type', '')}:{json.dumps(params, sort_keys=True)}"
        self._subscriptions.pop(key, None)
        await self.send(message)

    async def _send_raw(self, message: dict) -> None:
        """Send raw message to WebSocket."""
        if self._ws and not self._ws.closed:
            try:
                await self._ws.send_json(message)
            except Exception as e:
                logger.warning(f"Send failed: {e}")

    async def _receive_loop(self) -> None:
        """Main receive loop."""
        if not self._ws:
            return

        try:
            async for msg in self._ws:
                if msg.type == aiohttp.WSMsgType.TEXT:
                    try:
                        parsed = parse_message_in(msg.data)
                        self._emit(WsEvent(type=WsEventType.MESSAGE, message=parsed))
                    except Exception as e:
                        logger.warning(f"Message parse error: {e}")
                elif msg.type == aiohttp.WSMsgType.ERROR:
                    self._emit(WsEvent(type=WsEventType.ERROR, error=str(self._ws.exception())))
                elif msg.type in (aiohttp.WSMsgType.CLOSED, aiohttp.WSMsgType.CLOSING):
                    break
        except asyncio.CancelledError:
            return
        except Exception as e:
            logger.warning(f"Receive loop error: {e}")

        # Connection lost
        self._state = ReadyState.CLOSED
        self._emit(WsEvent(type=WsEventType.DISCONNECTED))

        if self._should_reconnect:
            await self._try_reconnect()

    async def _ping_loop(self) -> None:
        """Periodic ping to keep connection alive."""
        try:
            while self._state == ReadyState.OPEN:
                await asyncio.sleep(self._config.ping_interval_ms / 1000.0)
                if self._state == ReadyState.OPEN:
                    await self._send_raw(make_ping())
        except asyncio.CancelledError:
            return

    async def _try_reconnect(self) -> None:
        """Attempt reconnection with exponential backoff."""
        while (
            self._should_reconnect
            and self._reconnect_attempts < self._config.max_reconnect_attempts
        ):
            self._reconnect_attempts += 1
            delay = min(
                self._config.base_reconnect_delay_ms * (2 ** (self._reconnect_attempts - 1)),
                30_000,
            ) / 1000.0

            logger.info(
                f"Reconnecting in {delay:.1f}s "
                f"(attempt {self._reconnect_attempts}/{self._config.max_reconnect_attempts})"
            )
            await asyncio.sleep(delay)

            try:
                await self.connect()
                return
            except Exception:
                continue

        self._emit(WsEvent(type=WsEventType.MAX_RECONNECT_REACHED))


__all__ = ["WsClient"]
