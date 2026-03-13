"""Privy sub-client — embedded wallet RPC operations."""

from typing import TYPE_CHECKING

from ..http.retry import RetryPolicy
from . import (
    PrivyOrderEnvelope,
    SignAndSendTxResponse,
    ExportWalletResponse,
)

if TYPE_CHECKING:
    from ..http.client import LightconeHttp


class Privy:
    """Sub-client for Privy embedded wallet operations.

    Embedded wallets are provisioned during login by passing
    use_embedded_wallet=True to login_with_message(). All methods
    require an active authenticated session.
    """

    def __init__(self, http: "LightconeHttp"):
        self._http = http

    async def sign_and_send_tx(
        self,
        wallet_id: str,
        base64_tx: str,
    ) -> SignAndSendTxResponse:
        """Sign and send a Solana transaction via the user's Privy embedded wallet."""
        data = await self._http.post(
            "/api/privy/sign_and_send_tx",
            {"wallet_id": wallet_id, "base64_tx": base64_tx},
            retry_policy=RetryPolicy.NONE,
        )
        return SignAndSendTxResponse(hash=data.get("hash", ""))

    async def sign_and_send_order(
        self,
        wallet_id: str,
        order: PrivyOrderEnvelope,
    ) -> dict:
        """Sign an order hash via Privy and submit it to the exchange engine.

        The backend computes the order hash, signs via Privy, and submits
        the signed order internally — no round-trip back to the client.
        """
        return await self._http.post(
            "/api/privy/sign_and_send_order",
            {"wallet_id": wallet_id, "order": order.to_dict()},
            retry_policy=RetryPolicy.NONE,
        )

    async def sign_and_cancel_order(
        self,
        wallet_id: str,
        order_hash: str,
        maker: str,
    ) -> dict:
        """Cancel a limit order via Privy signing."""
        return await self._http.post(
            "/api/privy/sign_and_cancel_order",
            {
                "wallet_id": wallet_id,
                "maker": maker,
                "cancel_type": "limit",
                "order_hash": order_hash,
            },
            retry_policy=RetryPolicy.NONE,
        )

    async def sign_and_cancel_trigger_order(
        self,
        wallet_id: str,
        trigger_order_id: str,
        maker: str,
    ) -> dict:
        """Cancel a trigger order via Privy signing."""
        return await self._http.post(
            "/api/privy/sign_and_cancel_order",
            {
                "wallet_id": wallet_id,
                "maker": maker,
                "cancel_type": "trigger",
                "trigger_order_id": trigger_order_id,
            },
            retry_policy=RetryPolicy.NONE,
        )

    async def sign_and_cancel_all_orders(
        self,
        wallet_id: str,
        user_pubkey: str,
        orderbook_id: str,
        timestamp: int,
        salt: str,
    ) -> dict:
        """Cancel all orders for a user via Privy signing."""
        return await self._http.post(
            "/api/privy/sign_and_cancel_all_orders",
            {
                "wallet_id": wallet_id,
                "user_pubkey": user_pubkey,
                "orderbook_id": orderbook_id,
                "timestamp": timestamp,
                "salt": salt,
            },
            retry_policy=RetryPolicy.NONE,
        )

    async def export_wallet(
        self,
        wallet_id: str,
        decode_pubkey_base64: str,
    ) -> ExportWalletResponse:
        """Export an embedded wallet's private key (HPKE encrypted)."""
        data = await self._http.post(
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
