"""Order envelope builders for the Lightcone SDK.

Provides a fluent builder for ordinary signed orders.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from solders.keypair import Keypair
from solders.pubkey import Pubkey

from ..error import SigningError, ValidationError
from ..shared.scaling import (
    OrderbookRules,
    scale_price_size,
    validate_raw_amounts,
    validate_signed_fields,
)
from ..shared.types import (
    DepositSource,
    Side,
    SubmitOrderRequest,
    TimeInForce,
)
from .errors import MissingFieldError
from .orders import (
    _decode_side,
    apply_signature,
    generate_salt,
    hash_order_hex,
    sign_order,
    to_submit_request,
)
from .types import OrderSide, SignedOrder

if TYPE_CHECKING:
    from ..domain.orderbook import OrderBookPair


class LimitOrderEnvelope:
    """Fluent builder for limit orders.

    The salt is the order's only identity. When the caller does not set one,
    the first build draws a random u64 salt and every later ``payload()``,
    ``sign()``, ``finalize()``, or ``submit()`` on this envelope reuses it, so
    the hash a wallet signs is the hash that is submitted. Amounts derived
    from ``price()``/``size()`` are derived again on every build, so a later
    price, size, or side change is never ignored.

    # Example (human-readable price/size — exactly constructed with fetched rules)

        request = (LimitOrderEnvelope()
            .maker(maker_pubkey)
            .market(market_pubkey)
            .base_mint(yes_token)
            .quote_mint(usdc)
            .bid()
            .price("0.55")
            .size("100")
            .sign(keypair, orderbook, rules))

    # Example (pre-computed raw amounts — exact rules preflight)

        request = (LimitOrderEnvelope()
            .maker(maker_pubkey)
            .market(market_pubkey)
            .base_mint(yes_token)
            .quote_mint(usdc)
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .sign(keypair, orderbook, rules))
    """

    def __init__(self):
        self._salt: int | None = None
        self._maker: Pubkey | None = None
        self._market: Pubkey | None = None
        self._base_mint: Pubkey | None = None
        self._quote_mint: Pubkey | None = None
        self._side: OrderSide | None = None
        self._amount_in: int | None = None
        self._amount_out: int | None = None
        # Whether the amounts were derived from price and size by a build.
        self._amounts_derived: bool = False
        self._expiration: int = 0
        self._price_str: str | None = None
        self._size_str: str | None = None
        self._deposit_source: DepositSource | None = None
        self._time_in_force: TimeInForce | None = None

    def salt(self, salt: int) -> LimitOrderEnvelope:
        """Set the order's identity salt (any u64). Drawn once when omitted."""
        self._salt = salt
        return self

    def maker(self, maker: Pubkey) -> LimitOrderEnvelope:
        self._maker = maker
        return self

    def market(self, market: Pubkey) -> LimitOrderEnvelope:
        self._market = market
        return self

    def base_mint(self, mint: Pubkey) -> LimitOrderEnvelope:
        self._base_mint = mint
        return self

    def quote_mint(self, mint: Pubkey) -> LimitOrderEnvelope:
        self._quote_mint = mint
        return self

    def bid(self) -> LimitOrderEnvelope:
        self._clear_derived_amounts()
        self._side = OrderSide.BID
        return self

    def ask(self) -> LimitOrderEnvelope:
        self._clear_derived_amounts()
        self._side = OrderSide.ASK
        return self

    def side(self, side: Side) -> LimitOrderEnvelope:
        self._clear_derived_amounts()
        self._side = _decode_side(side)
        return self

    def amount_in(self, amount: int) -> LimitOrderEnvelope:
        """Set raw atoms the maker gives. Replaces amounts derived from price and size."""
        self._clear_derived_amounts()
        self._amount_in = amount
        return self

    def amount_out(self, amount: int) -> LimitOrderEnvelope:
        """Set raw atoms the maker receives. Replaces amounts derived from price and size."""
        self._clear_derived_amounts()
        self._amount_out = amount
        return self

    def expiration(self, expiration: int) -> LimitOrderEnvelope:
        """Set the expiration in Unix seconds (0 for no expiration)."""
        self._expiration = expiration
        return self

    def price(self, price: str) -> LimitOrderEnvelope:
        """Store an exact human-readable price for sign()/finalize()."""
        self._clear_derived_amounts()
        self._price_str = price
        return self

    def size(self, size: str) -> LimitOrderEnvelope:
        """Store an exact human-readable size for sign()/finalize()."""
        self._clear_derived_amounts()
        self._size_str = size
        return self

    def deposit_source(self, ds: DepositSource) -> LimitOrderEnvelope:
        """Set the deposit source for order matching."""
        self._deposit_source = ds
        return self

    def time_in_force(self, tif: TimeInForce) -> LimitOrderEnvelope:
        """Set time-in-force policy (GTC, IOC, or FOK)."""
        self._time_in_force = tif
        return self

    def _clear_derived_amounts(self) -> None:
        """Forget amounts derived from price and size by an earlier build."""
        if self._amounts_derived:
            self._amount_in = None
            self._amount_out = None
            self._amounts_derived = False

    def _auto_fill_from_orderbook(self, orderbook: OrderBookPair) -> None:
        """Fill market, mints, and salt from orderbook if not explicitly set."""
        if self._market is None:
            self._market = Pubkey.from_string(orderbook.market_pubkey)
        if self._salt is None:
            self._salt = generate_salt()
        if self._base_mint is None:
            self._base_mint = Pubkey.from_string(orderbook.base.pubkey)
        if self._quote_mint is None:
            self._quote_mint = Pubkey.from_string(orderbook.quote.pubkey)

    def _apply_rules(self, rules: OrderbookRules, orderbook_id: str) -> None:
        """Construct or preflight the exact signed ratio.

        Amounts derived from price and size are derived again on every build.
        """
        rules.validate_for_orderbook(orderbook_id)
        if self._side is None:
            raise MissingFieldError("side (call .bid() or .ask())")
        self._clear_derived_amounts()
        if self._amount_in is not None and self._amount_out is not None:
            validate_raw_amounts(
                self._amount_in, self._amount_out, int(self._side), rules
            )
            return
        if self._amount_in is not None or self._amount_out is not None:
            raise MissingFieldError(
                "amount_in and amount_out must be supplied together"
            )
        if self._price_str is None or self._size_str is None:
            raise MissingFieldError(
                "either price()+size() or amount_in()+amount_out() is required"
            )
        scaled = scale_price_size(
            self._price_str, self._size_str, int(self._side), rules
        )
        self._amount_in = scaled.amount_in
        self._amount_out = scaled.amount_out
        self._amounts_derived = True

    def payload(self) -> SignedOrder:
        """Build an unsigned SignedOrder without consuming the envelope.

        Draws the salt on first use and reuses it afterwards.
        """
        if self._maker is None:
            raise MissingFieldError("maker")
        if self._market is None:
            raise MissingFieldError("market")
        if self._base_mint is None:
            raise MissingFieldError("base_mint")
        if self._quote_mint is None:
            raise MissingFieldError("quote_mint")
        if self._side is None:
            raise MissingFieldError("side (call .bid() or .ask())")
        if self._amount_in is None:
            raise MissingFieldError("amount_in")
        if self._amount_out is None:
            raise MissingFieldError("amount_out")
        if self._amount_in == 0:
            raise MissingFieldError("amount_in must be greater than 0")
        if self._amount_out == 0:
            raise MissingFieldError("amount_out must be greater than 0")
        validate_signed_fields(self._amount_in, self._amount_out)
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

    def finalize(
        self, sig_bs58: str, orderbook: OrderBookPair, rules: OrderbookRules
    ) -> SubmitOrderRequest:
        """Apply an external wallet-adapter signature and produce a SubmitOrderRequest.

        Fetched rules are mandatory. Human values are constructed exactly and
        raw amounts are preflighted against the same admission rules.
        """
        self._auto_fill_from_orderbook(orderbook)
        self._apply_rules(rules, orderbook.orderbook_id)
        order = self.payload()
        apply_signature(order, sig_bs58, rules)
        return to_submit_request(
            order,
            orderbook.orderbook_id,
            time_in_force=self._time_in_force,
            deposit_source=self._deposit_source,
        )

    def sign(
        self, keypair: Keypair, orderbook: OrderBookPair, rules: OrderbookRules
    ) -> SubmitOrderRequest:
        """Sign and produce a SubmitOrderRequest.

        Fetched rules are mandatory. Human values are constructed exactly and
        raw amounts are preflighted against the same admission rules.
        """
        self._auto_fill_from_orderbook(orderbook)
        self._apply_rules(rules, orderbook.orderbook_id)
        order = self.payload()
        sign_order(order, keypair, rules)
        return to_submit_request(
            order,
            orderbook.orderbook_id,
            time_in_force=self._time_in_force,
            deposit_source=self._deposit_source,
        )

    # Field accessors (matching Rust get_* methods)

    @property
    def get_maker(self) -> Pubkey | None:
        return self._maker

    @property
    def get_market(self) -> Pubkey | None:
        return self._market

    @property
    def get_base_mint(self) -> Pubkey | None:
        return self._base_mint

    @property
    def get_quote_mint(self) -> Pubkey | None:
        return self._quote_mint

    @property
    def get_side(self) -> OrderSide | None:
        return self._side

    @property
    def get_amount_in(self) -> int | None:
        return self._amount_in

    @property
    def get_amount_out(self) -> int | None:
        return self._amount_out

    @property
    def get_expiration(self) -> int:
        return self._expiration

    @property
    def get_salt(self) -> int | None:
        """The salt, once set by the caller or fixed by the first build."""
        return self._salt

    @property
    def get_deposit_source(self) -> DepositSource | None:
        return self._deposit_source

    @property
    def get_time_in_force(self) -> TimeInForce | None:
        return self._time_in_force

    # ── Unified submit (dispatches based on client signing strategy) ──

    async def submit(self, client: object, orderbook: OrderBookPair):
        """Submit this order using the client's signing strategy.

        - **Native**: signs locally with the keypair, submits via REST
        - **WalletAdapter**: signs the order hash via the external signer,
          submits via REST
        - **Privy**: not supported for orders; raises ``ValidationError``

        Fills orderbook-derived fields (market, mints, salt) and scales
        price/size to raw amounts before signing. Submission is never retried:
        if the outcome is unknown (``ENGINE_UNAVAILABLE``, a timeout, or a lost
        connection), call ``submit`` again on this same envelope, which reuses
        its salt and resubmits the identical order; ``DUPLICATE_ORDER`` or
        ``ALREADY_EXISTS`` then proves the first submission was accepted.
        Changing any field, including the salt, creates a different order.

        Args:
            client: A ``LightconeClient`` instance with a signing strategy set.
            orderbook: The ``OrderBookPair`` for this order.

        Returns:
            ``SubmitOrderResponse`` on success.
        """
        import base58

        from ..shared.signing import SigningStrategyKind, classify_signer_error

        rules = await client.orderbooks().decimals(orderbook.orderbook_id)  # type: ignore[attr-defined]
        strategy = client._require_signing_strategy()  # type: ignore[attr-defined]
        # Pre-fill orderbook-derived fields and validate before signing
        self._auto_fill_from_orderbook(orderbook)
        self._apply_rules(rules, orderbook.orderbook_id)

        if strategy.kind == SigningStrategyKind.NATIVE:
            request = self.sign(strategy.keypair, orderbook, rules)
            return await client.orders().submit(request)  # type: ignore[attr-defined]

        if strategy.kind == SigningStrategyKind.WALLET_ADAPTER:
            order_hash = hash_order_hex(self.payload())
            try:
                sig_bytes = await strategy.signer.sign_message(
                    order_hash.encode("ascii")
                )
            except Exception as exc:
                raise classify_signer_error(str(exc)) from exc
            sig_bs58 = base58.b58encode(bytes(sig_bytes)).decode("ascii")
            request = self.finalize(sig_bs58, orderbook, rules)
            return await client.orders().submit(request)  # type: ignore[attr-defined]

        if strategy.kind == SigningStrategyKind.PRIVY:
            raise ValidationError(
                "Privy order signing is not supported; configure a native signer "
                "or an ExternalSigner"
            )

        raise SigningError(f"Unsupported signing strategy: {strategy.kind}")
