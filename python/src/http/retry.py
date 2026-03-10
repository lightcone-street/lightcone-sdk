"""Retry logic for HTTP requests.

Matches TS http/retry.ts with RetryPolicy, RetryConfig, and delay calculation.
"""

import random
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional, Set


class RetryPolicy(str, Enum):
    """Retry policy for HTTP requests."""

    NONE = "none"
    IDEMPOTENT = "idempotent"
    CUSTOM = "custom"


@dataclass
class RetryConfig:
    """Configuration for retry behavior."""

    max_retries: int = 3
    initial_delay_ms: int = 200
    max_delay_ms: int = 10_000
    backoff_factor: float = 2.0
    jitter: bool = True
    retryable_statuses: Set[int] = field(default_factory=lambda: {429, 500, 502, 503, 504})

    @staticmethod
    def default() -> "RetryConfig":
        """Default retry config: 3 retries, 200ms initial, 10s max, jitter on."""
        return RetryConfig()

    @staticmethod
    def none() -> "RetryConfig":
        """No retries."""
        return RetryConfig(max_retries=0)

    @staticmethod
    def idempotent() -> "RetryConfig":
        """Idempotent retry config: includes 429 rate limiting."""
        return RetryConfig(
            max_retries=3,
            initial_delay_ms=200,
            max_delay_ms=10_000,
            retryable_statuses={429, 500, 502, 503, 504},
        )


DEFAULT_RETRY_CONFIG = RetryConfig.default()


def delay_for_attempt(attempt: int, config: RetryConfig) -> float:
    """Calculate delay in seconds for a given retry attempt.

    Uses exponential backoff with optional jitter.

    Args:
        attempt: Zero-based attempt number (0 = first retry)
        config: Retry configuration

    Returns:
        Delay in seconds
    """
    delay_ms = config.initial_delay_ms * (config.backoff_factor ** attempt)
    delay_ms = min(delay_ms, config.max_delay_ms)

    if config.jitter:
        delay_ms = delay_ms * (0.5 + random.random() * 0.5)

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
