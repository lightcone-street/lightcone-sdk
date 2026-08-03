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
        "DUPLICATE_ORDER": "Duplicate Order",
        "POST_ONLY_WOULD_CROSS": "Post Only Would Cross",
        "FOK_NO_FILL": "FOK No Fill",
        "IOC_NO_FILL": "IOC No Fill",
        "WOULD_CROSS_UNAVAILABLE_LIQUIDITY": "Would Cross Unavailable Liquidity",
        "WOULD_CROSS_BOOK": "Would Cross Book",
        "MARKET_NOT_FOUND": "Market Not Found",
        "ORDERBOOK_NOT_FOUND": "Orderbook Not Found",
        "TOKEN_PAIR_MISMATCH": "Token Pair Mismatch",
        "INSUFFICIENT_MARKET_FEE_BUFFER": "Insufficient Market Fee Buffer",
        "SIGNATURE_EXPIRED": "Signature Expired",
        "TRADING_RULES_UNAVAILABLE": "Trading Rules Unavailable",
        "ORDER_FIELD_OUT_OF_RANGE": "Order Field Out of Range",
        "PRICE_NOT_EXACTLY_REPRESENTABLE": "Price Not Exactly Representable",
        "PRICE_OUT_OF_RANGE": "Price Out of Range",
        "INVALID_PRICE_DECIMALS": "Invalid Price Decimals",
        "INVALID_PRICE_SIGNIFICANT_FIGURES": "Invalid Price Significant Figures",
        "INVALID_SIZE_DECIMALS": "Invalid Size Decimals",
        "TRIGGER_PRICE_OUT_OF_RANGE": "Trigger Price Out of Range",
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
