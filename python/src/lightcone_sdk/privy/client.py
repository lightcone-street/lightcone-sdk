"""Privy sub-client — embedded wallet RPC operations."""

from __future__ import annotations

from typing import TYPE_CHECKING

from ..error import SdkError
from ..http.retry import RetryPolicy
from . import (
    ExportWalletResponse,
    SignAndSendTxResponse,
)

if TYPE_CHECKING:
    from ..client import LightconeClient


class Privy:
    """Sub-client for Privy embedded wallet operations.

    Embedded wallets are provisioned during login by passing
    use_embedded_wallet=True to login_with_message(). All methods
    require an active authenticated session.
    """

    def __init__(self, client: LightconeClient):
        self._client = client

    async def sign_and_send_tx(
        self,
        wallet_id: str,
        base64_tx: str,
    ) -> SignAndSendTxResponse:
        """Reject backend sends that cannot expose their signed v1 message."""
        raise SdkError(
            "Privy transaction signing cannot return verifiable v1 signed bytes; use a v1-capable external signer"
        )

    async def export_wallet(
        self,
        wallet_id: str,
        decode_pubkey_base64: str,
    ) -> ExportWalletResponse:
        """Export an embedded wallet's private key (HPKE encrypted)."""
        data = await self._client._http.post(
            "/api/privy/wallet/export",
            {
                "wallet_id": wallet_id,
                "decode_pubkey_base64": decode_pubkey_base64,
            },
            retry_policy=RetryPolicy.NONE,
        )
        return ExportWalletResponse(
            encryption_type=data.get("encryption_type", ""),
            ciphertext=data.get("ciphertext", ""),
            encapsulated_key=data.get("encapsulated_key", ""),
        )


__all__ = ["Privy"]
