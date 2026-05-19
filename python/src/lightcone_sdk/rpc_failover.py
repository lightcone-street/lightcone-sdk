"""RPC failover: automatic switch to a backup Solana RPC endpoint on
infrastructure errors, with 120 s cooldown recovery to primary.

Mirrors rust/src/rpc_failover.rs.
"""

from __future__ import annotations

import asyncio
import time
from enum import Enum
from typing import Optional


FAST_RETRY_DELAY_SECS = 0.1
COOLDOWN_DURATION_SECS = 120.0


class ActiveRpc(Enum):
    PRIMARY = "primary"
    BACKUP = "backup"


class RpcFailoverState:
    """Tracks which RPC endpoint is active and cooldown state."""

    def __init__(self) -> None:
        self.active: ActiveRpc = ActiveRpc.PRIMARY
        self.flipped_to_backup_at: Optional[float] = None

    def maybe_recover_to_primary(self) -> None:
        """If on backup and cooldown has elapsed, flip back to primary."""
        if self.active == ActiveRpc.BACKUP and self.flipped_to_backup_at is not None:
            if time.monotonic() - self.flipped_to_backup_at >= COOLDOWN_DURATION_SECS:
                self.active = ActiveRpc.PRIMARY
                self.flipped_to_backup_at = None

    def flip_to_backup(self) -> None:
        self.active = ActiveRpc.BACKUP
        self.flipped_to_backup_at = time.monotonic()

    def flip_to_primary(self) -> None:
        self.active = ActiveRpc.PRIMARY
        self.flipped_to_backup_at = None


def is_infrastructure_error(exc: BaseException) -> bool:
    """Return True if the exception indicates an RPC infrastructure failure
    (connection error, timeout, 502/503/504) rather than an application error.
    """
    if isinstance(exc, (asyncio.TimeoutError, TimeoutError, ConnectionError, OSError)):
        return True

    try:
        import aiohttp
        if isinstance(exc, (aiohttp.ClientConnectorError, aiohttp.ServerTimeoutError)):
            return True
        if isinstance(exc, aiohttp.ClientResponseError):
            return exc.status in (502, 503, 504)
    except ImportError:
        pass

    try:
        import httpx
        if isinstance(exc, (httpx.ConnectError, httpx.ConnectTimeout, httpx.ReadTimeout)):
            return True
    except ImportError:
        pass

    return False


__all__ = [
    "ActiveRpc",
    "RpcFailoverState",
    "FAST_RETRY_DELAY_SECS",
    "COOLDOWN_DURATION_SECS",
    "is_infrastructure_error",
]
