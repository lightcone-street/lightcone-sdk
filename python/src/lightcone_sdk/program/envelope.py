"""Order envelope builders for the Lightcone SDK.

Provides fluent builder pattern for constructing limit and trigger orders.
"""

from __future__ import annotations

from decimal import Decimal
from typing import Optional, TYPE_CHECKING

from solders.keypair import Keypair
from solders.pubkey import Pubkey

from .types import SignedOrder, OrderSide
from .orders import sign_order, to_submit_request, apply_signature, signature_hex
from ..shared.types import (
    DepositSource,
    Side,
    SubmitOrderRequest,
    SubmitTriggerOrderRequest,
    TimeInForce,
    TriggerType,
)
from ..shared.scaling import align_price_to_tick, scale_price_size

if TYPE_CHECKING:
    from ..domain.orderbook import OrderBookPair


class LimitOrderEnvelope:
    """Fluent builder for limit orders.

    # Example (human-readable price/size — auto-scaled)

        request = (LimitOrderEnvelope()
            .maker(maker_pubkey)
            .market(market_pubkey)
            .base_mint(yes_token)
            .quote_mint(usdc)
            .bid()
            .nonce(5)
            .price("0.55")
            .size("100")
            .sign(keypair, orderbook))

    # Example (pre-computed raw amounts — no scaling)

        request = (LimitOrderEnvelope()
            .maker(maker_pubkey)
            .market(market_pubkey)
            .base_mint(yes_token)
            .quote_mint(usdc)
            .bid()
            .nonce(5)
            .amount_in(1_000_000)
            .amount_out(500_000)
            .sign(keypair, orderbook))
    """

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
        self._price_str: Optional[str] = None
        self._size_str: Optional[str] = None
        self._deposit_source: Optional[DepositSource] = None
        self._time_in_force: Optional[TimeInForce] = None

    def nonce(self, nonce: int) -> LimitOrderEnvelope:
        self._nonce = nonce
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
        """Store human-readable price for auto-scaling in sign()/finalize()."""
        self._price_str = price
        return self

    def size(self, size: str) -> LimitOrderEnvelope:
        """Store human-readable size for auto-scaling in sign()/finalize()."""
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

    def gtc(self) -> LimitOrderEnvelope:
        self._time_in_force = TimeInForce.GTC
        return self

    def ioc(self) -> LimitOrderEnvelope:
        self._time_in_force = TimeInForce.IOC
        return self

    def fok(self) -> LimitOrderEnvelope:
        self._time_in_force = TimeInForce.FOK
        return self

    def alo(self) -> LimitOrderEnvelope:
        """Set time-in-force to add-liquidity-only."""
        self._time_in_force = TimeInForce.ALO
        return self

    def _auto_scale(self, orderbook: OrderBookPair) -> None:
        """Auto-scale price/size to raw amounts if not already set.

        Skips if amount_in/amount_out are already non-zero (raw amounts
        were provided directly). Otherwise requires price() and size()
        to have been called.
        """
        if self._amount_in or self._amount_out:
            return

        assert self._price_str is not None, \
            "either price()+size() or amount_in()+amount_out() is required"
        assert self._size_str is not None, \
            "either price()+size() or amount_in()+amount_out() is required"

        decimals = orderbook.decimals()
        aligned_price = align_price_to_tick(Decimal(self._price_str), decimals)
        scaled = scale_price_size(str(aligned_price), self._size_str, int(self._side), decimals)
        self._amount_in = scaled.amount_in
        self._amount_out = scaled.amount_out

    def payload(self) -> SignedOrder:
        """Build an unsigned SignedOrder without consuming the envelope."""
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

    def finalize(self, sig_bs58: str, orderbook: OrderBookPair) -> SubmitOrderRequest:
        """Apply an external wallet-adapter signature and produce a SubmitOrderRequest.

        If price() and size() were set, scaling is applied automatically
        using the orderbook's decimals. If amount_in() and amount_out()
        were set directly, those raw values are used as-is.
        """
        self._auto_scale(orderbook)
        order = self.payload()
        apply_signature(order, sig_bs58)
        return to_submit_request(
            order, orderbook.orderbook_id,
            time_in_force=self._time_in_force,
            deposit_source=self._deposit_source,
        )

    def sign(self, keypair: Keypair, orderbook: OrderBookPair) -> SubmitOrderRequest:
        """Sign and produce a SubmitOrderRequest.

        If price() and size() were set, scaling is applied automatically
        using the orderbook's decimals. If amount_in() and amount_out()
        were set directly, those raw values are used as-is.
        """
        self._auto_scale(orderbook)
        order = self.payload()
        sign_order(order, keypair)
        return to_submit_request(
            order, orderbook.orderbook_id,
            time_in_force=self._time_in_force,
            deposit_source=self._deposit_source,
        )

    # Field accessors (matching Rust fields_* methods)

    @property
    def fields_maker(self) -> Optional[Pubkey]:
        return self._maker

    @property
    def fields_market(self) -> Optional[Pubkey]:
        return self._market

    @property
    def fields_base_mint(self) -> Optional[Pubkey]:
        return self._base_mint

    @property
    def fields_quote_mint(self) -> Optional[Pubkey]:
        return self._quote_mint

    @property
    def fields_side(self) -> Optional[OrderSide]:
        return self._side

    @property
    def fields_amount_in(self) -> Optional[int]:
        return self._amount_in

    @property
    def fields_amount_out(self) -> Optional[int]:
        return self._amount_out

    @property
    def fields_expiration(self) -> int:
        return self._expiration

    @property
    def fields_nonce(self) -> Optional[int]:
        return self._nonce

    @property
    def fields_deposit_source(self) -> Optional[DepositSource]:
        return self._deposit_source

    @property
    def fields_time_in_force(self) -> Optional[TimeInForce]:
        return self._time_in_force


class TriggerOrderEnvelope:
    """Fluent builder for trigger orders."""

    def __init__(self):
        self._limit = LimitOrderEnvelope()
        self._trigger_price: Optional[float] = None
        self._trigger_type: Optional[TriggerType] = None
        self._time_in_force: TimeInForce = TimeInForce.GTC

    def nonce(self, nonce: int) -> TriggerOrderEnvelope:
        self._limit.nonce(nonce)
        return self

    def maker(self, maker: Pubkey) -> TriggerOrderEnvelope:
        self._limit.maker(maker)
        return self

    def market(self, market: Pubkey) -> TriggerOrderEnvelope:
        self._limit.market(market)
        return self

    def base_mint(self, mint: Pubkey) -> TriggerOrderEnvelope:
        self._limit.base_mint(mint)
        return self

    def quote_mint(self, mint: Pubkey) -> TriggerOrderEnvelope:
        self._limit.quote_mint(mint)
        return self

    def bid(self) -> TriggerOrderEnvelope:
        self._limit.bid()
        return self

    def ask(self) -> TriggerOrderEnvelope:
        self._limit.ask()
        return self

    def side(self, side: Side) -> TriggerOrderEnvelope:
        self._limit.side(side)
        return self

    def amount_in(self, amount: int) -> TriggerOrderEnvelope:
        self._limit.amount_in(amount)
        return self

    def amount_out(self, amount: int) -> TriggerOrderEnvelope:
        self._limit.amount_out(amount)
        return self

    def expiration(self, expiration: int) -> TriggerOrderEnvelope:
        self._limit.expiration(expiration)
        return self

    def price(self, price: str) -> TriggerOrderEnvelope:
        self._limit.price(price)
        return self

    def size(self, size: str) -> TriggerOrderEnvelope:
        self._limit.size(size)
        return self

    def deposit_source(self, ds: DepositSource) -> TriggerOrderEnvelope:
        self._limit.deposit_source(ds)
        return self

    def trigger_price(self, price: float) -> TriggerOrderEnvelope:
        self._trigger_price = price
        return self

    def trigger_type(self, tt: TriggerType) -> TriggerOrderEnvelope:
        self._trigger_type = tt
        return self

    def stop_loss(self, price: float) -> TriggerOrderEnvelope:
        """Set trigger type to STOP_LOSS and trigger price."""
        self._trigger_type = TriggerType.STOP_LOSS
        self._trigger_price = price
        return self

    def take_profit(self, price: float) -> TriggerOrderEnvelope:
        """Set trigger type to TAKE_PROFIT and trigger price."""
        self._trigger_type = TriggerType.TAKE_PROFIT
        self._trigger_price = price
        return self

    def time_in_force(self, tif: TimeInForce) -> TriggerOrderEnvelope:
        self._time_in_force = tif
        return self

    def gtc(self) -> TriggerOrderEnvelope:
        self._time_in_force = TimeInForce.GTC
        return self

    def ioc(self) -> TriggerOrderEnvelope:
        self._time_in_force = TimeInForce.IOC
        return self

    def fok(self) -> TriggerOrderEnvelope:
        self._time_in_force = TimeInForce.FOK
        return self

    def alo(self) -> TriggerOrderEnvelope:
        """Set time-in-force to add-liquidity-only."""
        self._time_in_force = TimeInForce.ALO
        return self

    def payload(self) -> SignedOrder:
        """Build an unsigned SignedOrder without consuming the envelope."""
        return self._limit.payload()

    def finalize(self, sig_bs58: str, orderbook: OrderBookPair) -> SubmitOrderRequest:
        """Apply external signature and produce a SubmitOrderRequest.

        Same auto-scaling behavior as sign().
        """
        assert self._trigger_price is not None, "trigger_price is required for trigger orders"
        assert self._trigger_type is not None, "trigger_type is required for trigger orders"
        self._limit._auto_scale(orderbook)
        order = self.payload()
        apply_signature(order, sig_bs58)
        return to_submit_request(
            order,
            orderbook.orderbook_id,
            time_in_force=self._time_in_force,
            trigger_price=self._trigger_price,
            trigger_type=self._trigger_type,
            deposit_source=self._limit.fields_deposit_source,
        )

    def sign(self, keypair: Keypair, orderbook: OrderBookPair) -> SubmitOrderRequest:
        """Sign and produce a SubmitOrderRequest.

        Same auto-scaling behavior as LimitOrderEnvelope.sign().
        """
        assert self._trigger_price is not None, "trigger_price is required for trigger orders"
        assert self._trigger_type is not None, "trigger_type is required for trigger orders"
        self._limit._auto_scale(orderbook)
        order = self.payload()
        sign_order(order, keypair)
        return to_submit_request(
            order,
            orderbook.orderbook_id,
            time_in_force=self._time_in_force,
            trigger_price=self._trigger_price,
            trigger_type=self._trigger_type,
            deposit_source=self._limit.fields_deposit_source,
        )

    def to_submit_trigger_request(self, order: SignedOrder, orderbook_id: str) -> SubmitTriggerOrderRequest:
        """Convert to a SubmitTriggerOrderRequest."""
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
            trigger_price=str(self._trigger_price) if self._trigger_price is not None else "0",
            trigger_type=self._trigger_type or TriggerType.STOP_LOSS,
            time_in_force=self._time_in_force,
        )

    # Field accessors

    @property
    def fields_maker(self) -> Optional[Pubkey]:
        return self._limit.fields_maker

    @property
    def fields_market(self) -> Optional[Pubkey]:
        return self._limit.fields_market

    @property
    def fields_base_mint(self) -> Optional[Pubkey]:
        return self._limit.fields_base_mint

    @property
    def fields_quote_mint(self) -> Optional[Pubkey]:
        return self._limit.fields_quote_mint

    @property
    def fields_side(self) -> Optional[OrderSide]:
        return self._limit.fields_side

    @property
    def fields_amount_in(self) -> Optional[int]:
        return self._limit.fields_amount_in

    @property
    def fields_amount_out(self) -> Optional[int]:
        return self._limit.fields_amount_out

    @property
    def fields_expiration(self) -> int:
        return self._limit.fields_expiration

    @property
    def fields_nonce(self) -> Optional[int]:
        return self._limit.fields_nonce

    @property
    def fields_deposit_source(self) -> Optional[DepositSource]:
        return self._limit.fields_deposit_source

    @property
    def fields_time_in_force(self) -> Optional[TimeInForce]:
        return self._time_in_force

    @property
    def fields_trigger_price(self) -> Optional[float]:
        return self._trigger_price

    @property
    def fields_trigger_type(self) -> Optional[TriggerType]:
        return self._trigger_type
