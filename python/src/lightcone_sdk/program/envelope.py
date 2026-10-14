"""Order envelope builders for the Lightcone SDK.

Provides a fluent builder for ordinary signed orders.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from solders.keypair import Keypair
from solders.pubkey import Pubkey

from ..error import SigningError
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
from .orders import apply_signature, sign_order, to_submit_request
from .types import OrderSide, SignedOrder

if TYPE_CHECKING:
    from ..domain.orderbook import OrderBookPair


class LimitOrderEnvelope:
    """Fluent builder for limit orders.

    # Example (human-readable price/size — exactly constructed with fetched rules)

        request = (LimitOrderEnvelope()
            .maker(maker_pubkey)
            .market(market_pubkey)
            .base_mint(yes_token)
            .quote_mint(usdc)
            .bid()
            .nonce(5)
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
            .nonce(5)
            .amount_in(1_000_000)
            .amount_out(500_000)
            .sign(keypair, orderbook, rules))
    """

    def __init__(self):
        self._nonce: int | None = None
        self._salt: int | None = None
        self._maker: Pubkey | None = None
        self._market: Pubkey | None = None
        self._base_mint: Pubkey | None = None
        self._quote_mint: Pubkey | None = None
        self._side: OrderSide = OrderSide.BID
        self._amount_in: int | None = None
        self._amount_out: int | None = None
        self._expiration: int = 0
        self._price_str: str | None = None
        self._size_str: str | None = None
        self._deposit_source: DepositSource | None = None
        self._time_in_force: TimeInForce | None = None

    def nonce(self, nonce: int) -> LimitOrderEnvelope:
        self._nonce = nonce
        return self

    def salt(self, salt: int) -> LimitOrderEnvelope:
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
        self._side = OrderSide.BID
        return self

    def ask(self) -> LimitOrderEnvelope:
        self._side = OrderSide.ASK
        return self

    def side(self, side: Side) -> LimitOrderEnvelope:
        self._side = OrderSide(int(side))
        return self

    def amount_in(self, amount: int) -> LimitOrderEnvelope:
        self._amount_in = amount
        return self

    def amount_out(self, amount: int) -> LimitOrderEnvelope:
        self._amount_out = amount
        return self

    def expiration(self, expiration: int) -> LimitOrderEnvelope:
        self._expiration = expiration
        return self

    def price(self, price: str) -> LimitOrderEnvelope:
        """Store an exact human-readable price for sign()/finalize()."""
        self._price_str = price
        return self

    def size(self, size: str) -> LimitOrderEnvelope:
        """Store an exact human-readable size for sign()/finalize()."""
        self._size_str = size
        return self

    def deposit_source(self, ds: DepositSource) -> LimitOrderEnvelope:
        """Set the deposit source for order matching."""
        self._deposit_source = ds
        return self

    def time_in_force(self, tif: TimeInForce) -> LimitOrderEnvelope:
        """Set time-in-force policy (GTC, IOC, FOK, ALO)."""
        self._time_in_force = tif
        return self

    def _auto_fill_from_orderbook(self, orderbook: OrderBookPair) -> None:
        """Fill market, mints, and salt from orderbook if not explicitly set."""
        if self._market is None:
            self._market = Pubkey.from_string(orderbook.market_pubkey)
        if self._salt is None:
            from .orders import generate_salt as _gen_salt

            self._salt = _gen_salt()
        if self._base_mint is None:
            self._base_mint = Pubkey.from_string(orderbook.base.pubkey)
        if self._quote_mint is None:
            self._quote_mint = Pubkey.from_string(orderbook.quote.pubkey)

    def _apply_rules(self, rules: OrderbookRules, orderbook_id: str) -> None:
        """Construct or preflight the exact signed ratio."""
        rules.validate_for_orderbook(orderbook_id)
        if self._amount_in is not None and self._amount_out is not None:
            validate_raw_amounts(
                self._amount_in, self._amount_out, int(self._side), rules
            )
            return
        if self._amount_in is not None or self._amount_out is not None:
            raise ValueError("amount_in and amount_out must be supplied together")
        if self._price_str is None or self._size_str is None:
            raise ValueError(
                "either price()+size() or amount_in()+amount_out() is required"
            )
        scaled = scale_price_size(
            self._price_str, self._size_str, int(self._side), rules
        )
        self._amount_in = scaled.amount_in
        self._amount_out = scaled.amount_out

    def payload(self) -> SignedOrder:
        """Build an unsigned SignedOrder without consuming the envelope."""
        assert self._maker is not None, "maker is required"
        assert self._market is not None, "market is required"
        assert self._base_mint is not None, "base_mint is required"
        assert self._quote_mint is not None, "quote_mint is required"
        assert self._nonce is not None, "nonce is required"
        assert self._amount_in is not None, "amount_in is required"
        assert self._amount_out is not None, "amount_out is required"
        if self._salt is None:
            from .orders import generate_salt as _gen_salt

            self._salt = _gen_salt()
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
    def get_nonce(self) -> int | None:
        return self._nonce

    @property
    def get_salt(self) -> int:
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

        - **Native**: signs locally with keypair, submits via REST
        - **WalletAdapter**: signs via external signer, submits via REST
        - **Privy**: sends to backend for signing and submission

        Args:
            client: A ``LightconeClient`` instance with a signing strategy set.
            orderbook: The ``OrderBookPair`` for this order.

        Returns:
            ``SubmitOrderResponse`` on success.
        """
        from ..shared.signing import SigningStrategyKind, classify_signer_error

        rules = await client.orderbooks().decimals(orderbook.orderbook_id)  # type: ignore[attr-defined]
        # Pre-fill orderbook-derived fields and validate before signing
        self._auto_fill_from_orderbook(orderbook)
        self._apply_rules(rules, orderbook.orderbook_id)

        # Cache nonce if explicitly provided, or auto-populate from cache
        if self._nonce is not None:
            client.set_order_nonce(self._nonce)  # type: ignore[attr-defined]
        else:
            self._nonce = client.order_nonce or 0  # type: ignore[attr-defined]

        strategy = client._require_signing_strategy()  # type: ignore[attr-defined]

        if strategy.kind == SigningStrategyKind.NATIVE:
            request = self.sign(strategy.keypair, orderbook, rules)
            return await client.orders().submit(request)  # type: ignore[attr-defined]

        elif strategy.kind == SigningStrategyKind.WALLET_ADAPTER:
            hash_hex = self.payload().hash_hex()
            try:
                sig_bytes = await strategy.signer.sign_message(hash_hex.encode())
            except Exception as exc:
                raise classify_signer_error(str(exc)) from exc
            import bs58 as _bs58

            sig_bs58 = _bs58.b58encode(sig_bytes).decode("ascii")
            request = self.finalize(sig_bs58, orderbook, rules)
            return await client.orders().submit(request)  # type: ignore[attr-defined]

        elif strategy.kind == SigningStrategyKind.PRIVY:
            from ..privy import privy_order_from_limit_envelope

            envelope = privy_order_from_limit_envelope(self, orderbook)
            result = await client.privy().sign_and_send_order(  # type: ignore[attr-defined]
                strategy.wallet_id,
                envelope,
            )
            from ..domain.order.convert import submit_response_from_dict

            return submit_response_from_dict(result)

        raise SigningError(f"Unsupported signing strategy: {strategy.kind}")
