"""Orderbook wire-to-domain conversion."""

from typing import TYPE_CHECKING

from .wire import DecimalsResponse, OrderbookResponse
from ...shared.scaling import OrderbookDecimals
from . import OrderBookPair, OrderBookValidationError

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
    base = next((t for t in conditional_tokens if t.pubkey == wire.base_token), None)
    quote = next((t for t in conditional_tokens if t.pubkey == wire.quote_token), None)

    errors: list[str] = []
    if base is None:
        errors.append(
            f"Base token not found: orderbook={wire.orderbook_id}, base={wire.base_token}"
        )
    if quote is None:
        errors.append(
            f"Quote token not found: orderbook={wire.orderbook_id}, quote={wire.quote_token}"
        )
    if errors:
        raise OrderBookValidationError("; ".join(errors))

    return OrderBookPair(
        id=wire.id,
        market_pubkey=wire.market_pubkey,
        orderbook_id=wire.orderbook_id,
        base=base,  # type: ignore[arg-type]
        quote=quote,  # type: ignore[arg-type]
        outcome_index=wire.outcome_index if wire.outcome_index else base.outcome_index,  # type: ignore[union-attr]
        tick_size=int(wire.tick_size) if wire.tick_size is not None else 0,
        total_bids=wire.total_bids,
        total_asks=wire.total_asks,
        last_trade_price=wire.last_trade_price,
        last_trade_time=wire.last_trade_time,
        active=wire.active,
    )
