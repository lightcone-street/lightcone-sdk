"""Position wire-to-domain conversion."""

from . import Position, PositionOutcome
from .wire import PositionEntryWire


def position_from_wire(wire: PositionEntryWire) -> Position:
    outcomes = [
        PositionOutcome(
            condition_id=o.outcome_index,
            condition_name="",
            token_mint=o.conditional_token,
            amount=int(o.balance_idle) + int(o.balance_on_book),
        )
        for o in wire.outcomes
    ]
    return Position(
        event_pubkey=wire.market_pubkey,
        outcomes=outcomes,
        created_at=wire.created_at,
    )
