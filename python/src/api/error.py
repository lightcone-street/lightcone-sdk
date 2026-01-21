"""API error types for the Lightcone REST API client."""

from dataclasses import dataclass
from typing import Optional


class ApiError(Exception):
    """Base exception for API errors."""

    pass


class HttpError(ApiError):
    """HTTP/network error."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"HTTP error: {message}")


class NotFoundError(ApiError):
    """Resource not found (404)."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Not found: {message}")


class BadRequestError(ApiError):
    """Invalid request parameters (400)."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Bad request: {message}")


class ForbiddenError(ApiError):
    """Permission denied (403)."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Permission denied: {message}")


class ConflictError(ApiError):
    """Resource already exists (409)."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Conflict: {message}")


class ServerError(ApiError):
    """Server-side error (5xx)."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Server error: {message}")


class DeserializeError(ApiError):
    """JSON deserialization error."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Deserialization error: {message}")


class InvalidParameterError(ApiError):
    """Invalid parameter provided."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(f"Invalid parameter: {message}")


class UnexpectedStatusError(ApiError):
    """Unexpected HTTP status code."""

    def __init__(self, status: int, message: str):
        self.status = status
        self.message = message
        super().__init__(f"Unexpected status {status}: {message}")


@dataclass
class ErrorResponse:
    """Error response format from the API."""

    status: Optional[str] = None
    message: Optional[str] = None
    details: Optional[str] = None

    def get_message(self) -> str:
        """Get the error message, preferring message over details."""
        return self.message or self.details or "Unknown error"

    @classmethod
    def from_dict(cls, data: dict) -> "ErrorResponse":
        """Create from dictionary."""
        return cls(
            status=data.get("status"),
            message=data.get("message") or data.get("error"),
            details=data.get("details"),
        )
