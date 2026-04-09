"""Machine-readable rejection codes from the backend API."""

from __future__ import annotations


class RejectionCode(str):
    """Backend rejection code with a human-readable label.

    Unknown codes are preserved verbatim for forward compatibility.
    """

    _LABELS = {
        "INSUFFICIENT_BALANCE": "Insufficient Balance",
        "EXPIRED": "Expired",
        "NONCE_MISMATCH": "Nonce Mismatch",
        "SELF_TRADE": "Self Trade",
        "MARKET_INACTIVE": "Market Inactive",
        "BELOW_MIN_ORDER_SIZE": "Below Min Order Size",
        "INVALID_NONCE": "Invalid Nonce",
        "BROADCAST_FAILURE": "Broadcast Failure",
        "ORDER_NOT_FOUND": "Order Not Found",
        "NOT_ORDER_MAKER": "Not Order Maker",
        "ORDER_ALREADY_FILLED": "Order Already Filled",
        "ORDER_ALREADY_CANCELLED": "Order Already Cancelled",
    }

    def __new__(cls, value: str) -> "RejectionCode":
        return super().__new__(cls, value)

    @classmethod
    def from_wire(cls, value: str | None) -> "RejectionCode | None":
        if value is None:
            return None
        normalized = value.upper()
        if normalized in cls._LABELS:
            return cls(normalized)
        return cls(value)

    @property
    def raw(self) -> str:
        return str.__str__(self)

    def normalized(self) -> str:
        return self.raw.upper()

    def is_known(self) -> bool:
        return self.normalized() in self._LABELS

    def label(self) -> str:
        return self._LABELS.get(self.normalized(), self.raw)

    def wire_name(self) -> str:
        return self.normalized() if self.is_known() else self.raw

    def __str__(self) -> str:
        return self.label()
