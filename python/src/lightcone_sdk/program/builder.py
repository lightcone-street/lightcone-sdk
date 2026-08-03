"""OrderBuilder - fluent API for constructing orders."""

from typing import Optional

from solders.keypair import Keypair
from solders.pubkey import Pubkey

from .types import SignedOrder, OrderSide
from .orders import sign_order, signature_hex, is_signed
from ..shared.types import Side, SubmitOrderRequest
from ..shared.scaling import (
    OrderbookRules,
    scale_price_size,
    validate_raw_amounts,
    validate_signed_fields,
)


class OrderBuilder:
    """Fluent builder for constructing SignedOrder instances."""

    def __init__(self):
        self._nonce: int = 0
        self._salt: int = 0
        self._maker: Optional[Pubkey] = None
        self._market: Optional[Pubkey] = None
        self._base_mint: Optional[Pubkey] = None
        self._quote_mint: Optional[Pubkey] = None
        self._side: OrderSide = OrderSide.BID
        self._amount_in: int = 0
        self._amount_out: int = 0
        self._expiration: int = 0
        self._rules: Optional[OrderbookRules] = None

    def nonce(self, nonce: int) -> "OrderBuilder":
        self._nonce = nonce
        return self

    def salt(self, salt: int) -> "OrderBuilder":
        self._salt = salt
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

    def rules(self, rules: OrderbookRules) -> "OrderBuilder":
        self._rules = rules
        return self

    def price(self, price: str, size: str, rules: OrderbookRules) -> "OrderBuilder":
        """Construct exact amounts under fetched immutable trading rules."""
        self._rules = rules
        scaled = scale_price_size(price, size, int(self._side), rules)
        self._amount_in = scaled.amount_in
        self._amount_out = scaled.amount_out
        return self

    def build(self, rules: Optional[OrderbookRules] = None) -> SignedOrder:
        """Build an unsigned SignedOrder."""
        assert self._maker is not None, "maker is required"
        assert self._market is not None, "market is required"
        assert self._base_mint is not None, "base_mint is required"
        assert self._quote_mint is not None, "quote_mint is required"
        resolved_rules = rules or self._rules
        assert resolved_rules is not None, "trading_rules are required"
        validate_raw_amounts(
            self._amount_in, self._amount_out, int(self._side), resolved_rules
        )
        validate_signed_fields(
            self._amount_in, self._amount_out, self._salt, self._nonce
        )

        return SignedOrder(
            nonce=self._nonce,
            salt=self._salt,
            maker=self._maker,
            market=self._market,
            base_mint=self._base_mint,
            quote_mint=self._quote_mint,
            side=self._side,
            amount_in=self._amount_in,
            amount_out=self._amount_out,
            expiration=self._expiration,
        )

    def build_and_sign(
        self, keypair: Keypair, rules: Optional[OrderbookRules] = None
    ) -> SignedOrder:
        """Build and sign an order."""
        resolved_rules = rules or self._rules
        assert resolved_rules is not None, "trading_rules are required"
        order = self.build(resolved_rules)
        sign_order(order, keypair, resolved_rules)
        return order

    def to_submit_request(
        self,
        keypair: Keypair,
        orderbook_id: str,
        rules: Optional[OrderbookRules] = None,
    ) -> SubmitOrderRequest:
        """Build, sign, and convert to an API submit request."""
        order = self.build_and_sign(keypair, rules)
        return SubmitOrderRequest(
            maker=str(order.maker),
            nonce=order.nonce,
            salt=order.salt,
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
