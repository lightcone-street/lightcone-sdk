"""Generic API response wrapper and structured rejection details."""

from __future__ import annotations

from dataclasses import dataclass, replace
from typing import Generic, Literal, Optional, TypeVar, cast

from .rejection import RejectionCode

T = TypeVar("T")
# Public login-method names exposed in bounded identity conflicts.
LinkedIdentityType = Literal["email", "google", "x", "wallet"]


@dataclass(frozen=True)
class ApiRejectedDetails:
    """Structured backend rejection with bounded method and transport context.

    Unknown future ``existing_method`` values become ``None`` so callers retain
    the stable rejection code and can fall back to generic guidance.
    """

    reason: str
    rejection_code: Optional[RejectionCode] = None
    error_code: Optional[str] = None
    existing_method: Optional[LinkedIdentityType] = None
    error_log_id: Optional[str] = None
    request_id: Optional[str] = None
    # HTTP status of the response that carried this rejection. Set by the HTTP
    # client when the rejection rode a non-2xx response; not present in the
    # backend JSON. Lets callers classify rejections at the transport level
    # (e.g. ``lightcone_sdk.error.is_unauthorized``) without matching on
    # backend error strings.
    http_status: Optional[int] = None

    @staticmethod
    def from_dict(data: dict) -> "ApiRejectedDetails":
        return ApiRejectedDetails(
            reason=str(data.get("reason", "")),
            rejection_code=RejectionCode.from_wire(data.get("rejection_code")),
            error_code=data.get("error_code"),
            existing_method=_linked_identity_type(data.get("existing_method")),
            error_log_id=data.get("error_log_id"),
        )

    def with_request_id(self, request_id: Optional[str]) -> "ApiRejectedDetails":
        return replace(self, request_id=request_id)

    def with_http_status(self, http_status: Optional[int]) -> "ApiRejectedDetails":
        return replace(self, http_status=http_status)

    def __str__(self) -> str:
        lines = [f"Reason: {self.reason}"]
        if self.rejection_code is not None:
            lines.append(f"Rejection Code: {self.rejection_code}")
        if self.error_code is not None:
            lines.append(f"Error Code: {self.error_code}")
        if self.existing_method is not None:
            lines.append(f"Existing Method: {self.existing_method}")
        if self.error_log_id is not None:
            lines.append(f"Error Log ID: {self.error_log_id}")
        if self.request_id is not None:
            lines.append(f"Request ID: {self.request_id}")
        return "\n".join(lines)


def _linked_identity_type(value: object) -> Optional[LinkedIdentityType]:
    """Keep known public method guidance and ignore future backend variants."""
    if value in ("email", "google", "x", "wallet"):
        return cast(LinkedIdentityType, value)
    return None


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
