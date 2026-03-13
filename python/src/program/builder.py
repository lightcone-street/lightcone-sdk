"""OrderBuilder - fluent API for constructing orders."""

from typing import Optional

from solders.keypair import Keypair
from solders.pubkey import Pubkey

from .types import SignedOrder, OrderSide
from .orders import sign_order, signature_hex, is_signed
from ..shared.types import Side, SubmitOrderRequest
from ..shared.scaling import OrderbookDecimals, scale_price_size


class OrderBuilder:
    """Fluent builder for constructing SignedOrder instances."""

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

    def nonce(self, nonce: int) -> "OrderBuilder":
        self._nonce = nonce
        return self

    def maker(self, maker: Pubkey) -> "OrderBuilder":
        self._maker = maker
        return self

    def market(self, market: Pubkey) -> "OrderBuilder":
        self._market = market
        return self

    def base_mint(self, mint: Pubkey) -> "OrderBuilder":
        self._base_mint = mint
        return self

    def quote_mint(self, mint: Pubkey) -> "OrderBuilder":
        self._quote_mint = mint
        return self

    def bid(self) -> "OrderBuilder":
        self._side = OrderSide.BID
        return self

    def ask(self) -> "OrderBuilder":
        self._side = OrderSide.ASK
        return self

    def side(self, side: Side) -> "OrderBuilder":
        self._side = OrderSide(int(side))
        return self

    def amount_in(self, amount: int) -> "OrderBuilder":
        self._amount_in = amount
        return self

    def amount_out(self, amount: int) -> "OrderBuilder":
        self._amount_out = amount
        return self

    def expiration(self, expiration: int) -> "OrderBuilder":
        self._expiration = expiration
        return self

    def price(self, price: str, size: str, decimals: OrderbookDecimals) -> "OrderBuilder":
        """Set amounts from price and size using decimal scaling."""
        scaled = scale_price_size(price, size, int(self._side), decimals)
        self._amount_in = scaled.amount_in
        self._amount_out = scaled.amount_out
        return self

    def build(self) -> SignedOrder:
        """Build an unsigned SignedOrder."""
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

    def build_and_sign(self, keypair: Keypair) -> SignedOrder:
        """Build and sign an order."""
        order = self.build()
        sign_order(order, keypair)
        return order

    def to_submit_request(self, keypair: Keypair, orderbook_id: str) -> SubmitOrderRequest:
        """Build, sign, and convert to an API submit request."""
        order = self.build_and_sign(keypair)
        return SubmitOrderRequest(
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
        )
