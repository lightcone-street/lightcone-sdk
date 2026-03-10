"""Order envelope builders for the Lightcone SDK.

Provides fluent builder pattern for constructing limit and trigger orders.
Matches TS program/envelope.ts.
"""

from typing import Optional

from solders.keypair import Keypair
from solders.pubkey import Pubkey

from .types import SignedOrder, OrderSide
from .orders import sign_order
from ..shared.types import Side, TimeInForce, TriggerType, SubmitTriggerOrderRequest
from ..shared.scaling import OrderbookDecimals, scale_price_size


class LimitOrderEnvelope:
    """Fluent builder for limit orders, matching TS LimitOrderEnvelope."""

    def __init__(self):
        self._nonce: int = 0
        self._maker: Optional[Pubkey] = None
        self._market: Optional[Pubkey] = None
        self._base_mint: Optional[Pubkey] = None
        self._quote_mint: Optional[Pubkey] = None
        self._side: OrderSide = OrderSide.BID
        self._amount_in: int = 0
        self._amount_out: int = 0
        self._expiration: int = 0

    def nonce(self, nonce: int) -> "LimitOrderEnvelope":
        self._nonce = nonce
        return self

    def maker(self, maker: Pubkey) -> "LimitOrderEnvelope":
        self._maker = maker
        return self

    def market(self, market: Pubkey) -> "LimitOrderEnvelope":
        self._market = market
        return self

    def base_mint(self, mint: Pubkey) -> "LimitOrderEnvelope":
        self._base_mint = mint
        return self

    def quote_mint(self, mint: Pubkey) -> "LimitOrderEnvelope":
        self._quote_mint = mint
        return self

    def bid(self) -> "LimitOrderEnvelope":
        self._side = OrderSide.BID
        return self

    def ask(self) -> "LimitOrderEnvelope":
        self._side = OrderSide.ASK
        return self

    def side(self, side: Side) -> "LimitOrderEnvelope":
        self._side = OrderSide(int(side))
        return self

    def amount_in(self, amount: int) -> "LimitOrderEnvelope":
        self._amount_in = amount
        return self

    def amount_out(self, amount: int) -> "LimitOrderEnvelope":
        self._amount_out = amount
        return self

    def expiration(self, expiration: int) -> "LimitOrderEnvelope":
        self._expiration = expiration
        return self

    def apply_scaling(self, price: str, size: str, decimals: OrderbookDecimals) -> "LimitOrderEnvelope":
        """Apply price/size scaling to set amount_in and amount_out."""
        scaled = scale_price_size(price, size, int(self._side), decimals)
        self._amount_in = scaled.amount_in
        self._amount_out = scaled.amount_out
        return self

    def finalize(self) -> SignedOrder:
        """Build the unsigned SignedOrder."""
        assert self._maker is not None, "maker is required"
        assert self._market is not None, "market is required"
        assert self._base_mint is not None, "base_mint is required"
        assert self._quote_mint is not None, "quote_mint is required"

        return SignedOrder(
            nonce=self._nonce,
            maker=self._maker,
            market=self._market,
            base_mint=self._base_mint,
            quote_mint=self._quote_mint,
            side=self._side,
            amount_in=self._amount_in,
            amount_out=self._amount_out,
            expiration=self._expiration,
        )

    def sign(self, keypair: Keypair) -> SignedOrder:
        """Build and sign the order."""
        order = self.finalize()
        sign_order(order, keypair)
        return order


class TriggerOrderEnvelope:
    """Fluent builder for trigger orders, matching TS TriggerOrderEnvelope."""

    def __init__(self):
        self._limit = LimitOrderEnvelope()
        self._trigger_price: str = "0"
        self._trigger_type: TriggerType = TriggerType.STOP_LOSS
        self._time_in_force: TimeInForce = TimeInForce.GTC

    def nonce(self, nonce: int) -> "TriggerOrderEnvelope":
        self._limit.nonce(nonce)
        return self

    def maker(self, maker: Pubkey) -> "TriggerOrderEnvelope":
        self._limit.maker(maker)
        return self

    def market(self, market: Pubkey) -> "TriggerOrderEnvelope":
        self._limit.market(market)
        return self

    def base_mint(self, mint: Pubkey) -> "TriggerOrderEnvelope":
        self._limit.base_mint(mint)
        return self

    def quote_mint(self, mint: Pubkey) -> "TriggerOrderEnvelope":
        self._limit.quote_mint(mint)
        return self

    def bid(self) -> "TriggerOrderEnvelope":
        self._limit.bid()
        return self

    def ask(self) -> "TriggerOrderEnvelope":
        self._limit.ask()
        return self

    def side(self, side: Side) -> "TriggerOrderEnvelope":
        self._limit.side(side)
        return self

    def amount_in(self, amount: int) -> "TriggerOrderEnvelope":
        self._limit.amount_in(amount)
        return self

    def amount_out(self, amount: int) -> "TriggerOrderEnvelope":
        self._limit.amount_out(amount)
        return self

    def expiration(self, expiration: int) -> "TriggerOrderEnvelope":
        self._limit.expiration(expiration)
        return self

    def apply_scaling(self, price: str, size: str, decimals: OrderbookDecimals) -> "TriggerOrderEnvelope":
        self._limit.apply_scaling(price, size, decimals)
        return self

    def trigger_price(self, price: str) -> "TriggerOrderEnvelope":
        self._trigger_price = price
        return self

    def trigger_type(self, tt: TriggerType) -> "TriggerOrderEnvelope":
        self._trigger_type = tt
        return self

    def stop_loss(self) -> "TriggerOrderEnvelope":
        self._trigger_type = TriggerType.STOP_LOSS
        return self

    def take_profit(self) -> "TriggerOrderEnvelope":
        self._trigger_type = TriggerType.TAKE_PROFIT
        return self

    def time_in_force(self, tif: TimeInForce) -> "TriggerOrderEnvelope":
        self._time_in_force = tif
        return self

    def gtc(self) -> "TriggerOrderEnvelope":
        self._time_in_force = TimeInForce.GTC
        return self

    def ioc(self) -> "TriggerOrderEnvelope":
        self._time_in_force = TimeInForce.IOC
        return self

    def fok(self) -> "TriggerOrderEnvelope":
        self._time_in_force = TimeInForce.FOK
        return self

    def sign(self, keypair: Keypair) -> SignedOrder:
        """Build and sign the underlying order."""
        return self._limit.sign(keypair)

    def to_submit_trigger_request(self, order: SignedOrder, orderbook_id: str) -> SubmitTriggerOrderRequest:
        """Convert to a SubmitTriggerOrderRequest."""
        from .orders import signature_hex
        return SubmitTriggerOrderRequest(
            maker=str(order.maker),
            nonce=order.nonce,
            market_pubkey=str(order.market),
            base_token=str(order.base_mint),
            quote_token=str(order.quote_mint),
            side=int(order.side),
            amount_in=order.amount_in,
            amount_out=order.amount_out,
            expiration=order.expiration,
            signature=signature_hex(order),
            orderbook_id=orderbook_id,
            trigger_price=self._trigger_price,
            trigger_type=int(self._trigger_type),
            time_in_force=int(self._time_in_force),
        )
