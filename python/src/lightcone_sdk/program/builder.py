"""OrderBuilder - fluent API for constructing orders."""

from solders.keypair import Keypair
from solders.pubkey import Pubkey

from ..shared.scaling import (
    OrderbookRules,
    scale_price_size,
    validate_raw_amounts,
    validate_signed_fields,
)
from ..shared.types import DepositSource, Side, SubmitOrderRequest
from .errors import MissingFieldError
from .orders import _decode_side, generate_salt, sign_order
from .orders import to_submit_request as order_to_submit_request
from .types import OrderSide, SignedOrder


class OrderBuilder:
    """Fluent builder for constructing SignedOrder instances.

    The salt is the order's only identity. When it is not set, the first
    ``build()`` draws one and every later build reuses it, so ``build()`` and
    ``build_and_sign()`` describe the same order.
    """

    def __init__(self):
        self._salt: int | None = None
        self._maker: Pubkey | None = None
        self._market: Pubkey | None = None
        self._base_mint: Pubkey | None = None
        self._quote_mint: Pubkey | None = None
        self._side: OrderSide = OrderSide.BID
        self._amount_in: int = 0
        self._amount_out: int = 0
        self._expiration: int = 0
        self._deposit_source: DepositSource | None = None
        self._rules: OrderbookRules | None = None
        # Price and size to scale at build time with the final side and rules.
        self._price_size: tuple[str, str] | None = None

    def salt(self, salt: int) -> "OrderBuilder":
        """Set the order's identity salt (any u64). Drawn once when omitted."""
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
        self._side = _decode_side(side)
        return self

    def amount_in(self, amount: int) -> "OrderBuilder":
        """Set amount in (raw atoms the maker gives). Replaces any price and size."""
        self._price_size = None
        self._amount_in = amount
        return self

    def amount_out(self, amount: int) -> "OrderBuilder":
        """Set amount out (raw atoms the maker receives). Replaces any price and size."""
        self._price_size = None
        self._amount_out = amount
        return self

    def expiration(self, expiration: int) -> "OrderBuilder":
        """Set the expiration in Unix seconds (0 for no expiration)."""
        self._expiration = expiration
        return self

    def deposit_source(self, source: DepositSource) -> "OrderBuilder":
        """Set the collateral source sent with the submit request."""
        self._deposit_source = source
        return self

    def rules(self, rules: OrderbookRules) -> "OrderBuilder":
        self._rules = rules
        return self

    def price(self, price: str, size: str, rules: OrderbookRules) -> "OrderBuilder":
        """Set price and size under fetched immutable trading rules.

        Every build constructs amount_in and amount_out exactly from them under
        the trading rules and the side set at build time, so
        ``.price(...).ask()`` and ``.ask().price(...)`` describe the same order.
        Invalid values are rejected here and at build, never rounded.
        """
        scale_price_size(price, size, int(self._side), rules)
        self._rules = rules
        self._price_size = (price, size)
        return self

    def build(self, rules: OrderbookRules | None = None) -> SignedOrder:
        """Build an unsigned SignedOrder."""
        if self._maker is None:
            raise MissingFieldError("maker")
        if self._market is None:
            raise MissingFieldError("market")
        if self._base_mint is None:
            raise MissingFieldError("base_mint")
        if self._quote_mint is None:
            raise MissingFieldError("quote_mint")
        resolved_rules = rules or self._rules
        if resolved_rules is None:
            raise MissingFieldError("trading_rules")
        if self._price_size is not None:
            price, size = self._price_size
            scaled = scale_price_size(price, size, int(self._side), resolved_rules)
            self._amount_in = scaled.amount_in
            self._amount_out = scaled.amount_out
        validate_signed_fields(self._amount_in, self._amount_out)
        validate_raw_amounts(
            self._amount_in, self._amount_out, int(self._side), resolved_rules
        )
        if self._salt is None:
            self._salt = generate_salt()

        return SignedOrder(
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
        self, keypair: Keypair, rules: OrderbookRules | None = None
    ) -> SignedOrder:
        """Build and sign an order."""
        resolved_rules = rules or self._rules
        if resolved_rules is None:
            raise MissingFieldError("trading_rules")
        order = self.build(resolved_rules)
        sign_order(order, keypair, resolved_rules)
        return order

    def to_submit_request(
        self,
        keypair: Keypair,
        orderbook_id: str,
        rules: OrderbookRules | None = None,
    ) -> SubmitOrderRequest:
        """Build, sign, and convert to an API submit request."""
        order = self.build_and_sign(keypair, rules)
        return order_to_submit_request(
            order, orderbook_id, deposit_source=self._deposit_source
        )
