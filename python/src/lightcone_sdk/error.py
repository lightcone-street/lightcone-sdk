"""Unified error types for the Lightcone SDK."""

from enum import Enum
from typing import Optional


class SdkError(Exception):
    """Base exception for all Lightcone SDK errors."""

    pass


class DeserializationError(SdkError):
    """Raised when a required field is missing during wire type deserialization."""

    pass


def _require(d: dict, key: str, type_name: str = ""):
    """Extract a required field from a dict, raising DeserializationError if missing."""
    if key not in d:
        ctx = f" in {type_name}" if type_name else ""
        raise DeserializationError(f"Missing required field '{key}'{ctx}")
    return d[key]


# ---------------------------------------------------------------------------
# HTTP Errors
# ---------------------------------------------------------------------------


class HttpErrorKind(str, Enum):
    """HTTP error variants."""

    REQUEST = "Request"
    SERVER_ERROR = "ServerError"
    RATE_LIMITED = "RateLimited"
    UNAUTHORIZED = "Unauthorized"
    NOT_FOUND = "NotFound"
    BAD_REQUEST = "BadRequest"
    TIMEOUT = "Timeout"
    MAX_RETRIES_EXCEEDED = "MaxRetriesExceeded"


class HttpError(SdkError):
    """HTTP/REST API error with typed variants."""

    def __init__(
        self,
        message: str,
        kind: HttpErrorKind = HttpErrorKind.REQUEST,
        status: Optional[int] = None,
        retry_after_ms: Optional[int] = None,
    ):
        super().__init__(message)
        self.kind = kind
        self.status = status
        self.retry_after_ms = retry_after_ms

    @staticmethod
    def request(message: str) -> "HttpError":
        return HttpError(message, HttpErrorKind.REQUEST)

    @staticmethod
    def server_error(message: str, status: int = 500) -> "HttpError":
        return HttpError(message, HttpErrorKind.SERVER_ERROR, status)

    @staticmethod
    def rate_limited(message: str = "Rate limited", retry_after_ms: Optional[int] = None) -> "HttpError":
        return HttpError(message, HttpErrorKind.RATE_LIMITED, 429, retry_after_ms=retry_after_ms)

    @staticmethod
    def unauthorized(message: str = "Unauthorized") -> "HttpError":
        return HttpError(message, HttpErrorKind.UNAUTHORIZED, 401)

    @staticmethod
    def not_found(message: str = "Not found") -> "HttpError":
        return HttpError(message, HttpErrorKind.NOT_FOUND, 404)

    @staticmethod
    def bad_request(message: str, status: int = 400) -> "HttpError":
        return HttpError(message, HttpErrorKind.BAD_REQUEST, status)

    @staticmethod
    def timeout(message: str = "Request timed out") -> "HttpError":
        return HttpError(message, HttpErrorKind.TIMEOUT)

    @staticmethod
    def max_retries_exceeded(attempts: int, last_error: str = "unknown") -> "HttpError":
        return HttpError(
            f"Max retries exceeded after {attempts} attempts: {last_error}",
            HttpErrorKind.MAX_RETRIES_EXCEEDED,
        )

    def is_retryable(self) -> bool:
        return self.kind in (
            HttpErrorKind.SERVER_ERROR,
            HttpErrorKind.RATE_LIMITED,
            HttpErrorKind.TIMEOUT,
        )


# ---------------------------------------------------------------------------
# WebSocket Errors
# ---------------------------------------------------------------------------


class WsErrorKind(str, Enum):
    """WebSocket error variants."""

    NOT_CONNECTED = "NotConnected"
    CONNECTION_FAILED = "ConnectionFailed"
    SEND_FAILED = "SendFailed"
    DESERIALIZATION_ERROR = "DeserializationError"
    PROTOCOL_ERROR = "ProtocolError"
    CLOSED = "Closed"


class WsError(SdkError):
    """WebSocket error with typed variants."""

    def __init__(
        self,
        message: str,
        kind: WsErrorKind = WsErrorKind.CONNECTION_FAILED,
        code: Optional[int] = None,
    ):
        super().__init__(message)
        self.kind = kind
        self.code = code

    @staticmethod
    def not_connected() -> "WsError":
        return WsError("Not connected", WsErrorKind.NOT_CONNECTED)

    @staticmethod
    def connection_failed(message: str) -> "WsError":
        return WsError(message, WsErrorKind.CONNECTION_FAILED)

    @staticmethod
    def send_failed(message: str) -> "WsError":
        return WsError(message, WsErrorKind.SEND_FAILED)

    @staticmethod
    def deserialization_error(message: str) -> "WsError":
        return WsError(message, WsErrorKind.DESERIALIZATION_ERROR)

    @staticmethod
    def protocol_error(message: str) -> "WsError":
        return WsError(message, WsErrorKind.PROTOCOL_ERROR)

    @staticmethod
    def closed(code: Optional[int] = None, reason: str = "") -> "WsError":
        msg = f"Connection closed: {reason}" if reason else "Connection closed"
        return WsError(msg, WsErrorKind.CLOSED, code)


# ---------------------------------------------------------------------------
# Auth Errors
# ---------------------------------------------------------------------------


class AuthErrorKind(str, Enum):
    """Authentication error variants."""

    NOT_AUTHENTICATED = "NotAuthenticated"
    LOGIN_FAILED = "LoginFailed"
    SIGNATURE_VERIFICATION_FAILED = "SignatureVerificationFailed"
    TOKEN_EXPIRED = "TokenExpired"


class AuthError(SdkError):
    """Authentication error with typed variants."""

    def __init__(
        self,
        message: str,
        kind: AuthErrorKind = AuthErrorKind.LOGIN_FAILED,
    ):
        super().__init__(message)
        self.kind = kind

    @staticmethod
    def not_authenticated() -> "AuthError":
        return AuthError("Not authenticated", AuthErrorKind.NOT_AUTHENTICATED)

    @staticmethod
    def login_failed(message: str) -> "AuthError":
        return AuthError(message, AuthErrorKind.LOGIN_FAILED)

    @staticmethod
    def signature_verification_failed(message: str = "Signature verification failed") -> "AuthError":
        return AuthError(message, AuthErrorKind.SIGNATURE_VERIFICATION_FAILED)

    @staticmethod
    def token_expired() -> "AuthError":
        return AuthError("Token expired", AuthErrorKind.TOKEN_EXPIRED)


__all__ = [
    "SdkError",
    "DeserializationError",
    "_require",
    "HttpError",
    "HttpErrorKind",
    "WsError",
    "WsErrorKind",
    "AuthError",
    "AuthErrorKind",
]
