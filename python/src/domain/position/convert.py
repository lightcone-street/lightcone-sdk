"""Position wire-to-domain conversion."""

from . import Position, PositionOutcome
from .wire import PositionEntryWire


def position_from_wire(wire: PositionEntryWire) -> Position:
    outcomes = [
        PositionOutcome(
            mint=o.conditional_token,
            name=o.name,
            amount=o.balance_idle + o.balance_on_book,
        )
        for o in wire.outcomes
    ]
    return Position(
        event_pubkey=wire.market_pubkey,
        outcomes=outcomes,
    )
