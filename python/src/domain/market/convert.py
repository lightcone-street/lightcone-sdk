"""Market wire-to-domain conversion."""

from typing import Optional
from . import (
    Market, Status, Outcome, ConditionalToken, DepositAsset,
    OrderBookPairSummary, TokenMetadata,
)
from .wire import MarketWire


def _parse_status(s: Optional[str]) -> Status:
    if s is None:
        return Status.PENDING
    try:
        return Status(s.lower())
    except ValueError:
        return Status.PENDING


def market_from_wire(wire: MarketWire) -> Market:
    """Convert a MarketWire to a Market domain type."""
    outcomes = [
        Outcome(
            name=o.get("name", ""),
            index=o.get("index", i),
            mint=o.get("mint"),
            symbol=o.get("symbol"),
        )
        for i, o in enumerate(wire.outcomes)
    ]

    conditional_tokens = [
        ConditionalToken(
            mint=ct.get("mint", ""),
            outcome_index=ct.get("outcome_index", 0),
            name=ct.get("name"),
            symbol=ct.get("symbol"),
            uri=ct.get("uri"),
            decimals=ct.get("decimals", 6),
        )
        for ct in wire.conditional_tokens
    ]

    deposit_assets = [
        DepositAsset(
            mint=da.get("mint", ""),
            symbol=da.get("symbol"),
            name=da.get("name"),
            decimals=da.get("decimals", 6),
            icon_url=da.get("icon_url"),
        )
        for da in wire.deposit_assets
    ]

    orderbook_pairs = [
        OrderBookPairSummary(
            id=ob.get("id", ""),
            base_token=ob.get("base_token", ""),
            quote_token=ob.get("quote_token", ""),
            outcome_index=ob.get("outcome_index", 0),
            tick_size=ob.get("tick_size"),
            active=ob.get("active", True),
        )
        for ob in wire.orderbook_pairs
    ]

    token_metadata = [
        TokenMetadata(
            mint=tm.get("mint", ""),
            name=tm.get("name"),
            symbol=tm.get("symbol"),
            decimals=tm.get("decimals", 6),
            icon_url=tm.get("icon_url"),
        )
        for tm in wire.token_metadata
    ]

    return Market(
        id=wire.id,
        pubkey=wire.pubkey,
        name=wire.name,
        slug=wire.slug,
        description=wire.description,
        status=_parse_status(wire.status),
        volume=wire.volume,
        outcomes=outcomes,
        conditional_tokens=conditional_tokens,
        deposit_assets=deposit_assets,
        orderbook_pairs=orderbook_pairs,
        token_metadata=token_metadata,
        icon_url=wire.icon_url,
        category=wire.category,
        featured=wire.featured,
        created_at=wire.created_at,
        resolved_at=wire.resolved_at,
        winning_outcome=wire.winning_outcome,
    )
