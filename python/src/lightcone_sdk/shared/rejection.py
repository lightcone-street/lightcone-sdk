"""Machine-readable rejection and error codes from the backend API.

Two independent code families ride the error envelope
(``{"status":"error","error_details":{...}}``):

- :class:`RejectionCode` (``rejection_code``): a business outcome of a trading
  mutation that the engine evaluated, delivered with HTTP 200.
- :class:`ErrorCode` (``error_code``): a transport, validation, authorization,
  or availability failure, delivered with a 4xx/5xx status.

Both keep an unrecognized code as its raw text, so a new backend code never
fails the surrounding error.
"""

from __future__ import annotations

_REJECTION_LABELS: dict[str, str] = {
    # This signed order identity was already accepted.
    "DUPLICATE_ORDER": "Duplicate Order",
    # Requested funding cannot cover the complete order.
    "INSUFFICIENT_BALANCE": "Insufficient Balance",
    # A fill-or-kill order's full base target is not executable.
    "FOK_INSUFFICIENT_LIQUIDITY": "FOK Insufficient Liquidity",
    # The order would cross the same wallet's executable liquidity.
    "SELF_TRADE": "Self Trade",
    # The order violates current trading rules (price/size precision, tick, or minimum size).
    "INVALID_ORDER": "Invalid Order",
    # The request exceeds supported limits.
    "INVALID_REQUEST": "Invalid Request",
    # The signed order had already expired.
    "ORDER_EXPIRED": "Order Expired",
    # Trading is paused for this market, exchange, or deposit token.
    "TRADING_PAUSED": "Trading Paused",
    # The canonical mutation signature did not verify.
    "INVALID_SIGNATURE": "Invalid Signature",
    # A cancel-all salt was already consumed.
    "CANCEL_ALL_REPLAY": "Cancel-All Replay",
    # Trading state changed while processing; the request may be retried.
    "STALE_STATE": "Stale State",
    # The request expired before acceptance; the request may be retried.
    "REQUEST_EXPIRED": "Request Expired",
    # Trading capacity is unavailable, including the per-wallet open-order cap.
    "TRADING_CAPACITY_UNAVAILABLE": "Trading Capacity Unavailable",
    # Committed trading state is not ready for this request yet.
    "TRADING_NOT_READY": "Trading Not Ready",
    # Committed order state could not be read.
    "INTERNAL_ERROR": "Internal Error",
    # The cancelled order hash was never accepted.
    "ORDER_NOT_FOUND": "Order Not Found",
    # The engine does not know the orderbook.
    "ORDERBOOK_NOT_FOUND": "Orderbook Not Found",
}

_TRANSIENT_REJECTIONS = frozenset(
    {
        "STALE_STATE",
        "REQUEST_EXPIRED",
        "TRADING_CAPACITY_UNAVAILABLE",
        "TRADING_NOT_READY",
    }
)


class RejectionCode(str):
    """Business rejection code for a trading mutation (HTTP 200 error envelope).

    Matches any case format (snake_case, SCREAMING_SNAKE_CASE). An unrecognized
    code keeps its raw text for forward compatibility.
    """

    _LABELS = _REJECTION_LABELS

    def __new__(cls, value: str) -> RejectionCode:
        return super().__new__(cls, value)

    @classmethod
    def from_wire(cls, value: str | None) -> RejectionCode | None:
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

    def known(self) -> str | None:
        """The known code, or ``None`` for a code this SDK version does not know."""
        normalized = self.normalized()
        return normalized if normalized in self._LABELS else None

    def is_known(self) -> bool:
        return self.known() is not None

    def is_transient(self) -> bool:
        """True when the order was not accepted because of a transient engine
        condition, so the same signed order may be submitted again (while it
        has not expired)."""
        return self.known() in _TRANSIENT_REJECTIONS

    def label(self) -> str:
        """Human-readable label (``INSUFFICIENT_BALANCE`` → ``Insufficient Balance``)."""
        return self._LABELS.get(self.normalized(), self.raw)

    def wire_name(self) -> str:
        """Wire format: SCREAMING_SNAKE_CASE for known codes, the raw text otherwise."""
        return self.known() or self.raw

    def __str__(self) -> str:
        return self.label()


_ERROR_CODES = frozenset(
    {
        # 400: malformed request field, bad side, zero amount, non-canonical key,
        # bad cancel-all salt, or stale cancel-all timestamp.
        "INVALID_ARGUMENT",
        # 400: engine precondition failed outside the business-rejection path.
        "FAILED_PRECONDITION",
        # 404.
        "NOT_FOUND",
        # 403: mutation signature rejected.
        "FORBIDDEN",
        # 409: duplicate order identity or reused cancel-all salt.
        "ALREADY_EXISTS",
        # 409: trading state changed; retry the request.
        "ABORTED",
        # 429: engine admission queue or capacity exhausted; retry later.
        "RESOURCE_EXHAUSTED",
        # 503: engine unavailable or deadline exceeded. For order submission the
        # outcome is unknown: resubmit the identical signed request, where
        # DUPLICATE_ORDER or ALREADY_EXISTS proves the first was accepted.
        "ENGINE_UNAVAILABLE",
        # 500: engine failure without a public reason.
        "ENGINE_INTERNAL_ERROR",
        # 503: committed trading state (or orderbook metadata) is temporarily
        # unavailable. Orders.submit fetches the orderbook rules first, so an
        # unknown orderbook fails there with ORDERBOOK_DECIMALS_NOT_FOUND.
        "TRADING_UNAVAILABLE",
        # 401: an order mutation had no authenticated session.
        "AUTH_REQUIRED",
        # 403: the session wallet does not match the order maker or cancellation wallet.
        "AUTH_WALLET_MISMATCH",
        # 400: unsupported time-in-force (only GTC, IOC and FOK are accepted).
        "INVALID_TIF",
        # 400: unsupported deposit source.
        "INVALID_DEPOSIT_SOURCE",
        # 400: signature is not 64 bytes of hex.
        "INVALID_SIGNATURE",
        # 400: a path or query public key is not canonical base58.
        "INVALID_PUBKEY",
        # 400: a pagination cursor is malformed or combined incorrectly.
        "INVALID_CURSOR",
        # 400: order hash is not 64 lowercase hex characters.
        "INVALID_ORDER_HASH",
        # 400: the route does not accept a query string.
        "UNEXPECTED_QUERY",
        # 400: a page limit is outside the route's accepted range.
        "INVALID_LIMIT",
        # 429: request rate limit exceeded.
        "RATE_LIMITED",
        # 404: no decimals (and so no trading rules) exist for the orderbook.
        "ORDERBOOK_DECIMALS_NOT_FOUND",
        # 500: the orderbook's stored trading rules are invalid.
        "ORDERBOOK_CONFIGURATION_ERROR",
        # 400: the route does not accept percent-encoded unreserved characters.
        "NON_CANONICAL_ENCODING",
    }
)

_RETRYABLE_ERROR_CODES = frozenset(
    {
        "ABORTED",
        "RESOURCE_EXHAUSTED",
        "ENGINE_UNAVAILABLE",
        "TRADING_UNAVAILABLE",
        "RATE_LIMITED",
    }
)


class ErrorCode(str):
    """Transport-level ``error_code`` carried by 4xx/5xx error envelopes.

    Matched exactly (codes are SCREAMING_SNAKE_CASE on the wire). An
    unrecognized code keeps its raw text.
    """

    def __new__(cls, value: str) -> ErrorCode:
        return super().__new__(cls, value)

    @classmethod
    def from_wire(cls, value: str | None) -> ErrorCode | None:
        return None if value is None else cls(value)

    @property
    def raw(self) -> str:
        return str.__str__(self)

    def known(self) -> str | None:
        """The known code, or ``None`` for a code this SDK version does not know."""
        return self.raw if self.raw in _ERROR_CODES else None

    def is_known(self) -> bool:
        return self.known() is not None

    def is_retryable(self) -> bool:
        """True for conditions that may clear on their own, where retrying the
        same request later is reasonable. For order submission,
        ``ENGINE_UNAVAILABLE`` additionally means the outcome is unknown: retry
        only the identical signed request, never a re-signed order."""
        return self.raw in _RETRYABLE_ERROR_CODES

    def wire_name(self) -> str:
        return self.raw


__all__ = ["ErrorCode", "RejectionCode"]
