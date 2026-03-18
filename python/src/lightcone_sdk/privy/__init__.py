"""Privy wallet integration types for the Lightcone SDK."""

from dataclasses import dataclass
from typing import Optional


@dataclass
class PrivyOrderEnvelope:
    """Wire type for the backend's Privy sign-and-send-order endpoint.

    Matches the backend's OrderForSigning struct.
    Prefer using from_limit() / from_trigger() over building manually.
    """

    maker: str = ""
    nonce: int = 0
    market_pubkey: str = ""
    base_token: str = ""
    quote_token: str = ""
    side: int = 0
    amount_in: int = 0
    amount_out: int = 0
    expiration: int = 0
    orderbook_id: str = ""
    time_in_force: Optional[str] = None
    trigger_price: Optional[float] = None
    trigger_type: Optional[str] = None

    def to_dict(self) -> dict:
        d: dict = {
            "maker": self.maker,
            "nonce": self.nonce,
            "market_pubkey": self.market_pubkey,
            "base_token": self.base_token,
            "quote_token": self.quote_token,
            "side": self.side,
            "amount_in": self.amount_in,
            "amount_out": self.amount_out,
            "expiration": self.expiration,
            "orderbook_id": self.orderbook_id,
        }
        if self.time_in_force is not None:
            d["tif"] = self.time_in_force
        if self.trigger_price is not None:
            d["trigger_price"] = self.trigger_price
        if self.trigger_type is not None:
            d["trigger_type"] = self.trigger_type
        return d


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
class SignAndSendOrderRequest:
    """Request to sign and send an order via Privy."""

    wallet_id: str = ""
    order: Optional[PrivyOrderEnvelope] = None


@dataclass
class SignAndCancelOrderRequest:
    """Request to cancel an order via Privy signing."""

    wallet_id: str = ""
    maker: str = ""
    cancel_type: str = ""  # "limit" or "trigger"
    order_hash: Optional[str] = None
    trigger_order_id: Optional[str] = None

    def to_dict(self) -> dict:
        d: dict = {
            "wallet_id": self.wallet_id,
            "maker": self.maker,
            "cancel_type": self.cancel_type,
        }
        if self.cancel_type == "limit" and self.order_hash is not None:
            d["order_hash"] = self.order_hash
        elif self.cancel_type == "trigger" and self.trigger_order_id is not None:
            d["trigger_order_id"] = self.trigger_order_id
        return d


@dataclass
class SignAndCancelAllRequest:
    """Request to cancel all orders via Privy signing."""

    wallet_id: str = ""
    user_pubkey: str = ""
    orderbook_id: str = ""
    timestamp: int = 0
    salt: str = ""


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


def privy_order_from_limit_envelope(envelope, orderbook) -> PrivyOrderEnvelope:
    """Build a PrivyOrderEnvelope from a LimitOrderEnvelope.

    Args:
        envelope: A LimitOrderEnvelope instance (from program.envelope)
        orderbook: The OrderBookPair for the order

    Returns:
        PrivyOrderEnvelope ready to send to the backend
    """
    order = envelope.payload()
    return PrivyOrderEnvelope(
        maker=str(order.maker),
        nonce=order.nonce,
        market_pubkey=str(order.market),
        base_token=str(order.base_mint),
        quote_token=str(order.quote_mint),
        side=int(order.side),
        amount_in=order.amount_in,
        amount_out=order.amount_out,
        expiration=order.expiration,
        orderbook_id=orderbook.orderbook_id,
    )


def privy_order_from_trigger_envelope(envelope, orderbook) -> PrivyOrderEnvelope:
    """Build a PrivyOrderEnvelope from a TriggerOrderEnvelope.

    Args:
        envelope: A TriggerOrderEnvelope instance (from program.envelope)
        orderbook: The OrderBookPair for the order

    Returns:
        PrivyOrderEnvelope with trigger fields populated
    """
    order = envelope.payload()

    trigger_price = None
    tp = envelope.fields_trigger_price
    if tp is not None and tp != 0:
        trigger_price = float(tp)

    trigger_type = None
    tt = envelope.fields_trigger_type
    if tt is not None:
        trigger_type = tt.as_wire()

    time_in_force = None
    tif = envelope.fields_time_in_force
    if tif is not None:
        time_in_force = tif.as_wire()

    return PrivyOrderEnvelope(
        maker=str(order.maker),
        nonce=order.nonce,
        market_pubkey=str(order.market),
        base_token=str(order.base_mint),
        quote_token=str(order.quote_mint),
        side=int(order.side),
        amount_in=order.amount_in,
        amount_out=order.amount_out,
        expiration=order.expiration,
        orderbook_id=orderbook.orderbook_id,
        time_in_force=time_in_force,
        trigger_price=trigger_price,
        trigger_type=trigger_type,
    )


__all__ = [
    "PrivyOrderEnvelope",
    "SignAndSendTxRequest",
    "SignAndSendTxResponse",
    "SignAndSendOrderRequest",
    "SignAndCancelOrderRequest",
    "SignAndCancelAllRequest",
    "ExportWalletRequest",
    "ExportWalletResponse",
    "privy_order_from_limit_envelope",
    "privy_order_from_trigger_envelope",
]
