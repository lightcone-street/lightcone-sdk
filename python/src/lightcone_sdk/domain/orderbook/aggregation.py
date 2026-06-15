"""Hyperliquid-style book aggregation parameters."""

from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class BookAggregation:
    """Book aggregation parameters for orderbook subscriptions and REST depth.

    ``(None, None)`` is full precision. ``n_sig_figs`` must be 2, 3, 4, or 5;
    ``mantissa`` must be 1, 2, or 5 and is only valid with ``n_sig_figs=5``.
    ``(5, None)`` normalizes to ``(5, 1)`` — the backend treats them as the
    same subscription, so all key/matching logic compares normalized values.

    Fields are snake_case like the rest of the SDK; the camelCase ``nSigFigs``
    spelling exists only at the wire boundary (subscribe params and REST query
    keys). Incoming ``book_update`` frames tag their view with snake_case
    ``n_sig_figs``/``mantissa``, omitted for full precision.
    """

    n_sig_figs: Optional[int] = None
    mantissa: Optional[int] = None

    @staticmethod
    def validate(
        n_sig_figs: Optional[int] = None, mantissa: Optional[int] = None
    ) -> "BookAggregation":
        """Validate against the backend's contract, returning the normalized
        form. Raises ``ValueError`` on invalid combinations — the server
        rejects them with ``INVALID_ORDERBOOK_SUBSCRIPTION`` (WS) or HTTP 400
        (REST), so validate before sending."""
        if n_sig_figs is None:
            if mantissa is not None:
                raise ValueError("mantissa is only valid when nSigFigs is 5")
            return FULL_PRECISION
        if n_sig_figs in (2, 3, 4):
            if mantissa is not None:
                raise ValueError("mantissa is only valid when nSigFigs is 5")
            return BookAggregation(n_sig_figs=n_sig_figs)
        if n_sig_figs == 5:
            if mantissa is None:
                return BookAggregation(n_sig_figs=5, mantissa=1)
            if mantissa in (1, 2, 5):
                return BookAggregation(n_sig_figs=5, mantissa=mantissa)
            raise ValueError("mantissa must be 1, 2, or 5")
        raise ValueError("nSigFigs must be 2, 3, 4, 5, or omitted")

    def normalized(self) -> "BookAggregation":
        """Normalized form: ``(5, None)`` becomes ``(5, 1)``; everything else
        is unchanged. Lenient — never raises. Use :meth:`validate` to reject
        invalid combinations before sending."""
        if self.n_sig_figs == 5 and self.mantissa is None:
            return BookAggregation(n_sig_figs=5, mantissa=1)
        return self

    @staticmethod
    def from_frame(
        n_sig_figs: Optional[int] = None, mantissa: Optional[int] = None
    ) -> "BookAggregation":
        """Aggregation identified by an incoming frame's tags. Untagged frames
        (both fields absent) are full precision. Lenient — never raises."""
        return BookAggregation(n_sig_figs=n_sig_figs, mantissa=mantissa).normalized()

    def is_full(self) -> bool:
        """Whether this is the full-precision (no aggregation) view."""
        return self.normalized() == FULL_PRECISION

    def key_suffix(self) -> str:
        """Stable suffix for subscription keys: ``"full"``,
        ``"sig2"``..``"sig4"``, or ``"sig5m1"``/``"sig5m2"``/``"sig5m5"``.
        Matches the backend's subscription-key vocabulary so keys are
        comparable across normalized spellings."""
        normalized = self.normalized()
        if normalized.n_sig_figs is None:
            return "full" if normalized.mantissa is None else "invalid"
        if normalized.mantissa is None:
            return f"sig{normalized.n_sig_figs}"
        return f"sig{normalized.n_sig_figs}m{normalized.mantissa}"


FULL_PRECISION = BookAggregation()
"""Full precision (no aggregation)."""


__all__ = [
    "BookAggregation",
    "FULL_PRECISION",
]
