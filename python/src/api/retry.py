"""Retry logic with exponential backoff for the API client."""

import asyncio
import random
from dataclasses import dataclass

from .error import (
    ServerError,
    RateLimitedError,
    HttpError,
)


@dataclass
class RetryConfig:
    """Configuration for retry behavior."""

    max_retries: int = 0  # 0 = disabled
    base_delay_ms: int = 100
    max_delay_ms: int = 10000

    @classmethod
    def default(cls) -> "RetryConfig":
        """Create default config (retry disabled)."""
        return cls()

    @classmethod
    def with_retries(cls, max_retries: int) -> "RetryConfig":
        """Create config with specified retry count."""
        return cls(max_retries=max_retries)

    def with_base_delay_ms(self, delay: int) -> "RetryConfig":
        """Set base delay."""
        self.base_delay_ms = delay
        return self

    def with_max_delay_ms(self, delay: int) -> "RetryConfig":
        """Set max delay cap."""
        self.max_delay_ms = delay
        return self


def is_retryable(error: Exception) -> bool:
    """Check if an error should trigger a retry."""
    if isinstance(error, ServerError):
        return True
    if isinstance(error, RateLimitedError):
        return True
    if isinstance(error, HttpError):
        return True
    # aiohttp connection errors
    if isinstance(error, (asyncio.TimeoutError, ConnectionError, OSError)):
        return True
    return False


def calculate_delay(attempt: int, config: RetryConfig) -> float:
    """Calculate delay with exponential backoff and jitter."""
    # Exponential backoff: base_delay * 2^attempt
    delay_ms = config.base_delay_ms * (2**attempt)

    # Cap at max_delay
    delay_ms = min(delay_ms, config.max_delay_ms)

    # Add jitter: 75-100% of calculated delay
    jitter = random.uniform(0.75, 1.0)
    delay_ms = int(delay_ms * jitter)

    return float(delay_ms) / 1000.0  # Convert to seconds
