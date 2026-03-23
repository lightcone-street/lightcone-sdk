"""Signing strategy types for client-level signing configuration.

The signing strategy determines how orders, cancels, and transactions
are signed. Set it on the client at construction time or update at runtime.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from enum import Enum
from typing import Optional

from ..error import SigningError, UserCancelled


class ExternalSigner(ABC):
    """Protocol for external wallet signers (browser wallet adapters).

    Implement this class to integrate a wallet adapter with the SDK.
    The SDK calls these methods internally when the signing strategy
    is ``WalletAdapter``.
    """

    @abstractmethod
    async def sign_message(self, message: bytes) -> bytes:
        """Sign a message and return the raw signature bytes."""
        ...

    @abstractmethod
    async def sign_transaction(self, tx_bytes: bytes) -> bytes:
        """Sign a serialized unsigned transaction and return the signed transaction bytes."""
        ...


class SigningStrategyKind(str, Enum):
    """Signing strategy type."""

    NATIVE = "native"
    WALLET_ADAPTER = "wallet_adapter"
    PRIVY = "privy"


class SigningStrategy:
    """Signing strategy for the client.

    Determines how orders, cancels, and transactions are signed.
    Set via builder methods or ``client.signing_strategy = ...`` at runtime.
    """

    def __init__(
        self,
        kind: SigningStrategyKind,
        keypair: object = None,
        signer: Optional[ExternalSigner] = None,
        wallet_id: Optional[str] = None,
    ):
        self.kind = kind
        self.keypair = keypair  # solders.keypair.Keypair (optional import)
        self.signer = signer
        self.wallet_id = wallet_id

    @staticmethod
    def native(keypair: object) -> "SigningStrategy":
        """Native keypair signing (CLI, bots).

        Signs locally using the provided keypair (``solders.keypair.Keypair``).
        """
        return SigningStrategy(
            kind=SigningStrategyKind.NATIVE,
            keypair=keypair,
        )

    @staticmethod
    def wallet_adapter(signer: ExternalSigner) -> "SigningStrategy":
        """External wallet adapter (browser).

        Delegates signing to the provided ``ExternalSigner`` implementation.
        """
        return SigningStrategy(
            kind=SigningStrategyKind.WALLET_ADAPTER,
            signer=signer,
        )

    @staticmethod
    def privy(wallet_id: str) -> "SigningStrategy":
        """Privy embedded wallet (backend-managed signing).

        The backend signs on behalf of the user using the Privy wallet.
        """
        return SigningStrategy(
            kind=SigningStrategyKind.PRIVY,
            wallet_id=wallet_id,
        )


# ── Rejection detection ──────────────────────────────────────────────────────

_CANCELLATION_KEYWORDS = (
    "reject",
    "cancel",
    "denied",
    "user refused",
    "declined",
    # wallet-adapter wraps JS rejection as InternalError with this message
    "reflect.get called on non-object",
)


def classify_signer_error(error: str) -> Exception:
    """Classify an external signer error string.

    Returns ``UserCancelled`` if the error indicates the user rejected
    the wallet popup, otherwise returns ``SigningError``.
    """
    lower = error.lower()
    for keyword in _CANCELLATION_KEYWORDS:
        if keyword in lower:
            return UserCancelled()
    return SigningError(error)


__all__ = [
    "ExternalSigner",
    "SigningStrategy",
    "SigningStrategyKind",
    "classify_signer_error",
]
