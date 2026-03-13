"""Position wire-to-domain conversion."""

from decimal import Decimal, InvalidOperation

from . import Position, PositionOutcome
from .wire import PositionEntryWire


def _sum_balances(idle: str, on_book: str) -> str:
    """Sum two decimal-string balances, returning a decimal string."""
    try:
        return str(Decimal(idle) + Decimal(on_book))
    except (InvalidOperation, ValueError):
        return "0"


def position_from_wire(wire: PositionEntryWire) -> Position:
    outcomes = [
        PositionOutcome(
            condition_id=o.outcome_index,
            condition_name="",
            token_mint=o.conditional_token,
            amount=o.balance or _sum_balances(o.balance_idle, o.balance_on_book),
        )
        for o in wire.outcomes
    ]
    return Position(
        event_pubkey=wire.market_pubkey,
        outcomes=outcomes,
        created_at=wire.created_at,
    )
