"""Generic API response wrapper and structured rejection details."""

from __future__ import annotations

from dataclasses import dataclass, replace
from typing import Generic, Optional, TypeVar

from .rejection import RejectionCode

T = TypeVar("T")


@dataclass(frozen=True)
class ApiRejectedDetails:
    """Structured rejection details returned by the backend."""

    reason: str
    rejection_code: Optional[RejectionCode] = None
    error_code: Optional[str] = None
    error_log_id: Optional[str] = None
    request_id: Optional[str] = None

    @staticmethod
    def from_dict(data: dict) -> "ApiRejectedDetails":
        return ApiRejectedDetails(
            reason=str(data.get("reason", "")),
            rejection_code=RejectionCode.from_wire(data.get("rejection_code")),
            error_code=data.get("error_code"),
            error_log_id=data.get("error_log_id"),
        )

    def with_request_id(self, request_id: Optional[str]) -> "ApiRejectedDetails":
        return replace(self, request_id=request_id)

    def __str__(self) -> str:
        lines = [f"Reason: {self.reason}"]
        if self.rejection_code is not None:
            lines.append(f"Rejection Code: {self.rejection_code}")
        if self.error_code is not None:
            lines.append(f"Error Code: {self.error_code}")
        if self.error_log_id is not None:
            lines.append(f"Error Log ID: {self.error_log_id}")
        if self.request_id is not None:
            lines.append(f"Request ID: {self.request_id}")
        return "\n".join(lines)


@dataclass(frozen=True)
class ApiResponse(Generic[T]):
    """Generic backend response wrapper.

    Success shape:
    ``{"status": "success", "body": ...}``

    Rejection shape:
    ``{"status": "error", "error_details": {...}}``
    """

    status: str
    body: Optional[T] = None
    details: Optional[ApiRejectedDetails] = None

    @classmethod
    def from_dict(cls, data: dict) -> "ApiResponse[T]":
        status = str(data.get("status", ""))
        if status == "success":
            return cls(status=status, body=data.get("body"))
        if status == "error":
            return cls(
                status=status,
                details=ApiRejectedDetails.from_dict(data.get("error_details") or {}),
            )
        raise ValueError(f"Unsupported ApiResponse status: {status}")
