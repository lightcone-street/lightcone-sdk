"""WebSocket-specific error types for the Lightcone SDK."""

from typing import Optional


class WebSocketError(Exception):
    """Base exception for WebSocket errors."""

    pass


class ConnectionFailedError(WebSocketError):
    """Initial connection failure."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Connection failed: {message}")


class ConnectionClosedError(WebSocketError):
    """Unexpected connection close."""

    def __init__(self, code: int, reason: str):
        self.code = code
        self.reason = reason
        super().__init__(f"Connection closed unexpectedly: code {code}, reason: {reason}")


class RateLimitedError(WebSocketError):
    """Rate limited (close code 1008)."""

    def __init__(self):
        super().__init__("Rate limited: too many connections from this IP")


class MessageParseError(WebSocketError):
    """JSON deserialization failure."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Failed to parse message: {message}")


class SequenceGapError(WebSocketError):
    """Detected sequence gap in book updates."""

    def __init__(self, expected: int, received: int):
        self.expected = expected
        self.received = received
        super().__init__(f"Sequence gap detected: expected {expected}, received {received}")


class ResyncRequiredError(WebSocketError):
    """Server requested resync."""

    def __init__(self, orderbook_id: str):
        self.orderbook_id = orderbook_id
        super().__init__(f"Resync required for orderbook: {orderbook_id}")


class SubscriptionFailedError(WebSocketError):
    """Subscription error from server."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Subscription failed: {message}")


class PingTimeoutError(WebSocketError):
    """Client ping not responded."""

    def __init__(self):
        super().__init__("Ping timeout: no pong response received")


class ProtocolError(WebSocketError):
    """WebSocket protocol error."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"WebSocket protocol error: {message}")


class ServerError(WebSocketError):
    """Server returned an error."""

    def __init__(self, code: str, message: str):
        self.code = code
        self.message = message
        super().__init__(f"Server error: {message} (code: {code})")


class NotConnectedError(WebSocketError):
    """Not connected to WebSocket server."""

    def __init__(self):
        super().__init__("Not connected to WebSocket server")


class AlreadyConnectedError(WebSocketError):
    """Already connected to WebSocket server."""

    def __init__(self):
        super().__init__("Already connected to WebSocket server")


class SendFailedError(WebSocketError):
    """Send failed."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Failed to send message: {message}")


class ChannelClosedError(WebSocketError):
    """Internal channel closed."""

    def __init__(self):
        super().__init__("Internal channel closed")


class InvalidUrlError(WebSocketError):
    """Invalid WebSocket URL."""

    def __init__(self, url: str):
        self.url = url
        super().__init__(f"Invalid WebSocket URL: {url}")


class OperationTimeoutError(WebSocketError):
    """Operation timed out."""

    def __init__(self):
        super().__init__("Operation timed out")


class AuthenticationFailedError(WebSocketError):
    """Authentication failed."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Authentication failed: {message}")


class AuthRequiredError(WebSocketError):
    """Authentication required for user stream."""

    def __init__(self):
        super().__init__("Authentication required for user stream")
