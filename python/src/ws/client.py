"""WebSocket client for the Lightcone SDK."""

import asyncio
import json
import logging
import random
from typing import Any, Callable, Optional

import aiohttp

from ..error import WsError
from . import (
    MessageIn,
    MessageInType,
    ReadyState,
    WsConfig,
    WsEvent,
    WsEventType,
    WS_DEFAULT_CONFIG,
    parse_message_in,
    ping as make_ping,
)
from .subscriptions import SubscribeParams, UserParams, subscription_key

logger = logging.getLogger(__name__)


class WsClient:
    """Async WebSocket client with reconnection and subscription tracking."""

    def __init__(self, config: Optional[WsConfig] = None):
        self._config = config or WS_DEFAULT_CONFIG
        self._session: Optional[aiohttp.ClientSession] = None
        self._ws: Optional[aiohttp.ClientWebSocketResponse] = None
        self._state = ReadyState.CLOSED
        self._callbacks: list[Callable[[WsEvent], Any]] = []
        self._active_subscriptions: list[SubscribeParams] = []
        self._reconnect_attempts = 0
        self._ping_task: Optional[asyncio.Task] = None
        self._pong_timeout_task: Optional[asyncio.Task] = None
        self._receive_task: Optional[asyncio.Task] = None
        self._should_reconnect = False
        self._auth_token: Optional[str] = None
        self._pending_messages: list[dict] = []
        self._awaiting_pong = False

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

            # Re-subscribe to tracked subscriptions
            for sub_params in self._active_subscriptions:
                msg = _subscribe_params_to_message(sub_params)
                await self._send_raw(msg)

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
                await self._try_reconnect(rate_limited=False)
            else:
                raise WsError.connection_failed(str(e))

    async def disconnect(self) -> None:
        """Disconnect from the WebSocket server."""
        self._should_reconnect = False
        self._state = ReadyState.CLOSING

        self._cancel_task("_ping_task")
        self._cancel_task("_pong_timeout_task")
        self._cancel_task("_receive_task")

        if self._ws and not self._ws.closed:
            await self._ws.close()

        if self._session and not self._session.closed:
            await self._session.close()

        self._ws = None
        self._session = None
        self._state = ReadyState.CLOSED
        self._emit(WsEvent(type=WsEventType.DISCONNECTED))

    async def restart_connection(self) -> None:
        """Force a fresh connection attempt.

        Tears down the current connection, resets the reconnect counter,
        and connects again.
        """
        if self._state == ReadyState.CONNECTING:
            logger.info("Already connecting, skipping restart")
            return

        logger.info("Manual reconnection requested")
        self._reconnect_attempts = 0
        await self.disconnect()
        self._should_reconnect = self._config.reconnect
        await self.connect()

    def clear_authed_subscriptions(self) -> None:
        """Remove authenticated subscriptions (e.g. User channel) from tracking."""
        before = len(self._active_subscriptions)
        self._active_subscriptions = [
            s for s in self._active_subscriptions
            if not isinstance(s, UserParams)
        ]
        removed = before - len(self._active_subscriptions)
        if removed > 0:
            logger.info(f"Cleared {removed} authenticated subscription(s)")

    async def send(self, message: dict) -> None:
        """Send a message. Queues if not connected."""
        if self._state == ReadyState.OPEN:
            await self._send_raw(message)
        else:
            self._pending_messages.append(message)

    async def subscribe(self, params: SubscribeParams) -> None:
        """Subscribe to a channel. Tracks subscription for reconnection."""
        # Track using SubscribeParams-based dedup
        if not any(subscription_key(s) == subscription_key(params) for s in self._active_subscriptions):
            self._active_subscriptions.append(params)

        message = _subscribe_params_to_message(params)
        await self.send(message)

    async def unsubscribe(self, params: SubscribeParams) -> None:
        """Unsubscribe from a channel."""
        key = subscription_key(params)
        self._active_subscriptions = [
            s for s in self._active_subscriptions
            if subscription_key(s) != key
        ]
        message = _unsubscribe_params_to_message(params)
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

        close_code: Optional[int] = None
        close_reason: str = ""

        try:
            async for msg in self._ws:
                if msg.type == aiohttp.WSMsgType.TEXT:
                    try:
                        parsed = parse_message_in(msg.data)
                        # Reset pong tracking on pong
                        if parsed.type == MessageInType.PONG:
                            self._awaiting_pong = False
                            self._cancel_task("_pong_timeout_task")
                            self._reconnect_attempts = 0
                        self._emit(WsEvent(type=WsEventType.MESSAGE, message=parsed))
                    except Exception as e:
                        logger.warning(f"Message parse error: {e}")
                elif msg.type == aiohttp.WSMsgType.ERROR:
                    self._emit(WsEvent(
                        type=WsEventType.ERROR,
                        error=str(self._ws.exception()),
                    ))
                elif msg.type in (aiohttp.WSMsgType.CLOSED, aiohttp.WSMsgType.CLOSING):
                    close_code = self._ws.close_code
                    close_reason = str(self._ws.close_code) if self._ws.close_code else ""
                    break
        except asyncio.CancelledError:
            return
        except Exception as e:
            logger.warning(f"Receive loop error: {e}")
            close_reason = str(e)

        # Connection lost
        self._state = ReadyState.CLOSED
        self._cancel_task("_ping_task")
        self._cancel_task("_pong_timeout_task")

        self._emit(WsEvent(
            type=WsEventType.DISCONNECTED,
            code=close_code,
            reason=close_reason,
        ))

        if self._should_reconnect:
            rate_limited = close_code == 1008
            await self._try_reconnect(rate_limited=rate_limited)

    async def _ping_loop(self) -> None:
        """Periodic ping to keep connection alive."""
        try:
            while self._state == ReadyState.OPEN:
                await asyncio.sleep(self._config.ping_interval_ms / 1000.0)
                if self._state == ReadyState.OPEN:
                    await self._send_raw(make_ping())
                    self._awaiting_pong = True
                    # Start pong timeout
                    self._cancel_task("_pong_timeout_task")
                    self._pong_timeout_task = asyncio.ensure_future(
                        self._pong_timeout_check()
                    )
        except asyncio.CancelledError:
            return

    async def _pong_timeout_check(self) -> None:
        """Check if pong was received within timeout."""
        try:
            await asyncio.sleep(self._config.pong_timeout_ms / 1000.0)
            if self._awaiting_pong and self._state == ReadyState.OPEN:
                logger.warning(
                    f"Pong timeout — no response within {self._config.pong_timeout_ms}ms"
                )
                self._emit(WsEvent(
                    type=WsEventType.DISCONNECTED,
                    reason="Pong timeout",
                ))
                # Force close and reconnect
                self._cancel_task("_ping_task")
                self._cancel_task("_receive_task")
                if self._ws and not self._ws.closed:
                    await self._ws.close()
                self._state = ReadyState.CLOSED
                if self._should_reconnect:
                    await self._try_reconnect(rate_limited=False)
        except asyncio.CancelledError:
            return

    async def _try_reconnect(self, rate_limited: bool = False) -> None:
        """Attempt reconnection with exponential backoff and jitter."""
        while (
            self._should_reconnect
            and self._reconnect_attempts < self._config.max_reconnect_attempts
        ):
            self._reconnect_attempts += 1
            self._state = ReadyState.CONNECTING

            exp = min(self._reconnect_attempts - 1, 10)
            base = self._config.base_reconnect_delay_ms * (2 ** exp)

            if rate_limited:
                jitter_max = 1000
                cap = 300_000  # 5 minutes
            else:
                jitter_max = 500
                cap = 60_000  # 60 seconds

            jitter = random.randint(0, jitter_max - 1)
            delay_ms = min(base + jitter, cap)
            delay = delay_ms / 1000.0

            logger.info(
                f"Reconnect attempt {self._reconnect_attempts}/"
                f"{self._config.max_reconnect_attempts} in {delay_ms}ms"
                f"{' (rate-limited)' if rate_limited else ''}"
            )
            await asyncio.sleep(delay)

            try:
                await self.connect()
                return
            except Exception:
                continue

        self._emit(WsEvent(type=WsEventType.MAX_RECONNECT_REACHED))

    def _cancel_task(self, attr: str) -> None:
        """Cancel and clear a named task attribute."""
        task = getattr(self, attr, None)
        if task is not None:
            task.cancel()
            setattr(self, attr, None)


def _subscribe_params_to_message(params: SubscribeParams) -> dict:
    """Convert SubscribeParams to a wire message dict."""
    from dataclasses import asdict
    d = asdict(params)
    # Remove include_ohlcv if False (default) for cleaner wire
    return {"method": "subscribe", "params": d}


def _unsubscribe_params_to_message(params: SubscribeParams) -> dict:
    """Convert SubscribeParams to an unsubscribe wire message dict."""
    from dataclasses import asdict
    d = asdict(params)
    # Remove fields not needed for unsubscribe
    d.pop("include_ohlcv", None)
    return {"method": "unsubscribe", "params": d}


__all__ = ["WsClient"]
