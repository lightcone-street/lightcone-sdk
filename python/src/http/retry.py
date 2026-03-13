"""Retry logic for HTTP requests."""

import random
from dataclasses import dataclass, field
from typing import Optional, Set


class RetryPolicy:
    """Retry policy for HTTP requests.

    Usage:
        RetryPolicy.NONE        — no retries
        RetryPolicy.IDEMPOTENT  — retry on transport errors + 429/502/503/504
        RetryPolicy.custom(cfg) — user-provided retry config
    """

    # Class-level constants set after class definition
    NONE: "RetryPolicy"
    IDEMPOTENT: "RetryPolicy"

    def __init__(self, _kind: str, _config: Optional["RetryConfig"] = None):
        self._kind = _kind
        self._config = _config

    @staticmethod
    def custom(config: "RetryConfig") -> "RetryPolicy":
        """Create a custom retry policy with the given config."""
        return RetryPolicy("custom", config)

    def is_none(self) -> bool:
        return self._kind == "none"

    def resolve_config(self) -> Optional["RetryConfig"]:
        """Resolve to a RetryConfig, or None for no retries.

        None  → no retry at all (skip the retry loop).
        Idempotent → RetryConfig.idempotent().
        Custom(c) → the user-provided config.
        """
        if self._kind == "none":
            return None
        if self._kind == "idempotent":
            return RetryConfig.idempotent()
        if self._kind == "custom":
            return self._config
        return None

    def __repr__(self) -> str:
        if self._kind == "custom":
            return f"RetryPolicy.custom({self._config!r})"
        return f"RetryPolicy.{self._kind.upper()}"

    def __eq__(self, other: object) -> bool:
        if isinstance(other, RetryPolicy):
            return self._kind == other._kind
        return NotImplemented

    def __hash__(self) -> int:
        return hash(self._kind)


RetryPolicy.NONE = RetryPolicy("none")
RetryPolicy.IDEMPOTENT = RetryPolicy("idempotent")


@dataclass
class RetryConfig:
    """Configuration for retry behavior."""

    max_retries: int = 3
    initial_delay_ms: int = 200
    max_delay_ms: int = 10_000
    backoff_factor: float = 2.0
    jitter: bool = True
    retryable_statuses: Set[int] = field(default_factory=lambda: {502, 503, 504})

    @staticmethod
    def default() -> "RetryConfig":
        """Default retry config: 3 retries, 200ms initial, 10s max.

        Retries on 502/503/504 (gateway errors).
        """
        return RetryConfig()

    @staticmethod
    def none() -> "RetryConfig":
        """No retries."""
        return RetryConfig(max_retries=0)

    @staticmethod
    def idempotent() -> "RetryConfig":
        """Idempotent retry config for GET requests.

        Retries on 429/502/503/504.
        """
        return RetryConfig(
            max_retries=3,
            initial_delay_ms=200,
            max_delay_ms=10_000,
            retryable_statuses={429, 502, 503, 504},
        )


DEFAULT_RETRY_CONFIG = RetryConfig.default()


def delay_for_attempt(attempt: int, config: RetryConfig) -> float:
    """Calculate delay in seconds for a given retry attempt.

    Uses exponential backoff with optional ±25% jitter.

    Args:
        attempt: Zero-based attempt number (0 = first retry)
        config: Retry configuration

    Returns:
        Delay in seconds
    """
    delay_ms = config.initial_delay_ms * (config.backoff_factor ** attempt)
    delay_ms = min(delay_ms, config.max_delay_ms)

    if config.jitter:
        jitter_range = delay_ms * 0.25
        jitter = (random.random() - 0.5) * 2.0 * jitter_range
        delay_ms = max(0.0, delay_ms + jitter)

    return delay_ms / 1000.0


def is_retryable_status(status: int, config: RetryConfig) -> bool:
    """Check if an HTTP status code is retryable."""
    return status in config.retryable_statuses


__all__ = [
    "RetryPolicy",
    "RetryConfig",
    "DEFAULT_RETRY_CONFIG",
    "delay_for_attempt",
    "is_retryable_status",
]
