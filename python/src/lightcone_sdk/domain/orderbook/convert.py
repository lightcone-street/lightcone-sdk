"""Orderbook wire-to-domain conversion."""

from typing import TYPE_CHECKING

from .wire import DecimalsResponse, OrderbookResponse
from ...shared.scaling import OrderbookDecimals
from . import (
    OrderBookPair,
    OrderBookValidationError,
    BaseTokenNotFound,
    QuoteTokenNotFound,
)

if TYPE_CHECKING:
    from ..market import ConditionalToken


def decimals_from_wire(wire: DecimalsResponse) -> OrderbookDecimals:
    return OrderbookDecimals(
        orderbook_id=wire.orderbook_id,
        base_decimals=wire.base_decimals,
        quote_decimals=wire.quote_decimals,
        price_decimals=wire.price_decimals,
    )


def orderbook_pair_from_wire(
    wire: OrderbookResponse,
    conditional_tokens: list["ConditionalToken"],
) -> OrderBookPair:
    """Convert an OrderbookResponse wire type to an OrderBookPair domain type.

    Matches Rust TryFrom<(OrderbookResponse, &Vec<ConditionalToken>)> for OrderBookPair.
    """
    base = next((t for t in conditional_tokens if t.mint == wire.base_token), None)
    quote = next((t for t in conditional_tokens if t.mint == wire.quote_token), None)

    errors: list[OrderBookValidationError] = []
    if base is None:
        errors.append(
            BaseTokenNotFound(
                f"orderbook: {wire.orderbook_id}, base: {wire.base_token}"
            )
        )
    if quote is None:
        errors.append(
            QuoteTokenNotFound(
                f"orderbook: {wire.orderbook_id}, quote: {wire.quote_token}"
            )
        )
    if errors:
        raise OrderBookValidationError(
            f"OrderBook validation errors ({wire.orderbook_id}): "
            + "; ".join(str(e) for e in errors)
        )

    # After the errors check above, base and quote are guaranteed non-None.
    # Use ok_or-style assertions matching Rust's defensive pattern.
    if base is None:
        raise BaseTokenNotFound(f"orderbook: {wire.orderbook_id}")
    if quote is None:
        raise QuoteTokenNotFound(f"orderbook: {wire.orderbook_id}")

    return OrderBookPair(
        id=wire.id,
        market_pubkey=wire.market_pubkey,
        orderbook_id=wire.orderbook_id,
        base=base,
        quote=quote,
        outcome_index=wire.outcome_index if wire.outcome_index else base.outcome_index,
        tick_size=int(wire.tick_size) if wire.tick_size is not None else 0,
        total_bids=wire.total_bids,
        total_asks=wire.total_asks,
        last_trade_price=wire.last_trade_price,
        last_trade_time=wire.last_trade_time,
        active=wire.active,
    )
