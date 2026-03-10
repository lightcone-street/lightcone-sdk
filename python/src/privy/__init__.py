"""Privy wallet integration types for the Lightcone SDK.

Matches TS privy/index.ts with order envelope conversion
and transaction signing types.
"""

from dataclasses import dataclass
from typing import Optional


@dataclass
class PrivyOrderEnvelope:
    """Order data formatted for Privy wallet signing.

    Amounts are u32 (not u64/bigint) since Privy wallets
    work with smaller numeric ranges.
    """

    nonce: int
    market: str
    base_mint: str
    quote_mint: str
    side: int
    amount_in: int
    amount_out: int
    expiration: int

    def to_dict(self) -> dict:
        return {
            "nonce": self.nonce,
            "market": self.market,
            "base_mint": self.base_mint,
            "quote_mint": self.quote_mint,
            "side": self.side,
            "amount_in": self.amount_in,
            "amount_out": self.amount_out,
            "expiration": self.expiration,
        }


@dataclass
class SignAndSendTxRequest:
    """Request to sign and send a transaction via Privy."""

    wallet_id: str
    base64_tx: str


@dataclass
class SignAndSendTxResponse:
    """Response from Privy transaction signing."""

    signature: str
    success: bool


@dataclass
class SignAndSendOrderRequest:
    """Request to sign and send an order via Privy."""

    wallet_id: str
    order: PrivyOrderEnvelope
    orderbook_id: str


@dataclass
class ExportWalletRequest:
    """Request to export a Privy wallet."""

    wallet_id: str
    decode_pubkey_base64: Optional[str] = None


@dataclass
class ExportWalletResponse:
    """Response from wallet export."""

    encrypted_key: str
    pubkey: str


def privy_order_from_limit_envelope(envelope) -> PrivyOrderEnvelope:
    """Convert a LimitOrderEnvelope to a PrivyOrderEnvelope.

    Args:
        envelope: A LimitOrderEnvelope instance (from program.envelope)

    Returns:
        PrivyOrderEnvelope with string pubkey fields
    """
    order = envelope.finalize()
    return PrivyOrderEnvelope(
        nonce=order.nonce,
        market=str(order.market),
        base_mint=str(order.base_mint),
        quote_mint=str(order.quote_mint),
        side=int(order.side),
        amount_in=order.amount_in,
        amount_out=order.amount_out,
        expiration=order.expiration,
    )


def privy_order_from_trigger_envelope(envelope) -> PrivyOrderEnvelope:
    """Convert a TriggerOrderEnvelope to a PrivyOrderEnvelope.

    Args:
        envelope: A TriggerOrderEnvelope instance (from program.envelope)

    Returns:
        PrivyOrderEnvelope with string pubkey fields
    """
    order = envelope._limit.finalize()
    return PrivyOrderEnvelope(
        nonce=order.nonce,
        market=str(order.market),
        base_mint=str(order.base_mint),
        quote_mint=str(order.quote_mint),
        side=int(order.side),
        amount_in=order.amount_in,
        amount_out=order.amount_out,
        expiration=order.expiration,
    )


__all__ = [
    "PrivyOrderEnvelope",
    "SignAndSendTxRequest",
    "SignAndSendTxResponse",
    "SignAndSendOrderRequest",
    "ExportWalletRequest",
    "ExportWalletResponse",
    "privy_order_from_limit_envelope",
    "privy_order_from_trigger_envelope",
]
