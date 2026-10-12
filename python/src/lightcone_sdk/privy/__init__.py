"""Privy wallet integration types for the Lightcone SDK.

Privy order signing and cancellation are not supported: the committed backend
has no Privy order route. Submit and cancel orders with a native signer or an
``ExternalSigner``.
"""

from dataclasses import dataclass


@dataclass
class SignAndSendTxRequest:
    """Request to sign and send a transaction via Privy."""

    wallet_id: str = ""
    base64_tx: str = ""


@dataclass
class SignAndSendTxResponse:
    """Response from Privy transaction signing."""

    hash: str = ""


@dataclass
class ExportWalletRequest:
    """Request to export a Privy wallet."""

    wallet_id: str = ""
    decode_pubkey_base64: str = ""


@dataclass
class ExportWalletResponse:
    """Response from wallet export (HPKE encrypted)."""

    encryption_type: str = ""
    ciphertext: str = ""
    encapsulated_key: str = ""


__all__ = [
    "SignAndSendTxRequest",
    "SignAndSendTxResponse",
    "ExportWalletRequest",
    "ExportWalletResponse",
]
