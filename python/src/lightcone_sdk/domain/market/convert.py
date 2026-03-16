"""Market wire-to-domain conversion."""

from typing import Optional
from . import (
    Market, Status, Outcome, ConditionalToken, DepositAsset,
    TokenMetadata,
)
from ..orderbook import OrderBookPair
from .wire import MarketWire


def _parse_status(s: Optional[str]) -> Status:
    if s is None:
        return Status.PENDING
    return Status.from_str(s)


def validation_errors_from_wire(wire: MarketWire) -> list[str]:
    errors: list[str] = []

    if not wire.slug:
        errors.append("Missing slug")
    if not wire.market_name:
        errors.append("Missing name")
    if not wire.description:
        errors.append("Missing description")
    if not wire.definition:
        errors.append("Missing definition")
    if not wire.icon_url:
        errors.append("Missing thumbnail image")
    if not wire.banner_image_url:
        errors.append("Missing banner image")
    if wire.market_status and wire.market_status not in {"Pending", "Active", "Resolved", "Cancelled"}:
        errors.append("Invalid status")

    if not errors:
        return []

    identifier = wire.market_pubkey or str(wire.market_id)
    return [f"Market validation errors ({identifier}): {', '.join(errors)}"]


def market_from_wire(wire: MarketWire) -> Market:
    """Convert a MarketWire to a Market domain type."""
    outcomes = [
        Outcome(
            index=o.index,
            name=o.name,
            icon_url=o.icon_url,
        )
        for o in wire.outcomes
    ]

    conditional_tokens: list[ConditionalToken] = []
    token_metadata: dict[str, TokenMetadata] = {}
    deposit_assets: list[DepositAsset] = []
    orderbook_ids: list[str] = []

    for da in wire.deposit_assets:
        da_symbol = da.token_symbol or da.symbol
        deposit_assets.append(DepositAsset(
            id=da.id,
            market_pda=da.market_pubkey,
            deposit_asset=da.deposit_asset,
            num_outcomes=da.num_outcomes,
            name=da.display_name,
            symbol=da_symbol,
            description=da.description,
            decimals=da.decimals,
            icon_url=da.icon_url,
        ))

        for ct in da.conditional_mints:
            ct_name = ct.display_name or ct.name
            ct_symbol = ct.short_name or ct.symbol
            conditional_tokens.append(ConditionalToken(
                mint=ct.token_address,
                outcome_index=ct.outcome_index,
                outcome=ct.outcome,
                deposit_asset=da.deposit_asset,
                deposit_symbol=da_symbol,
                name=ct_name,
                symbol=ct_symbol,
                description=ct.description,
                decimals=ct.decimals,
                icon_url=ct.icon_url,
            ))
            if ct.token_address:
                token_metadata[ct.token_address] = TokenMetadata(
                    pubkey=ct.token_address,
                    symbol=ct_symbol,
                    decimals=ct.decimals,
                    icon_url=ct.icon_url,
                    name=ct_name,
                )

    # Build a lookup from mint address to ConditionalToken for orderbook pairs.
    ct_by_mint: dict[str, ConditionalToken] = {ct.mint: ct for ct in conditional_tokens}

    orderbook_pairs = []
    for ob in wire.orderbooks:
        base_ct = ct_by_mint.get(ob.base_token, ConditionalToken(mint=ob.base_token, outcome_index=0))
        quote_ct = ct_by_mint.get(ob.quote_token, ConditionalToken(mint=ob.quote_token, outcome_index=0))
        orderbook_pairs.append(OrderBookPair(
            id=ob.id,
            market_pubkey=ob.market_pubkey or wire.market_pubkey,
            orderbook_id=ob.orderbook_id,
            base=base_ct,
            quote=quote_ct,
            outcome_index=ob.outcome_index,
            tick_size=int(ob.tick_size) if ob.tick_size is not None else 0,
            total_bids=ob.total_bids,
            total_asks=ob.total_asks,
            last_trade_price=ob.last_trade_price,
            last_trade_time=ob.last_trade_time,
            active=ob.active,
        ))
        if ob.orderbook_id:
            orderbook_ids.append(ob.orderbook_id)

    return Market(
        id=wire.market_id,
        pubkey=wire.market_pubkey,
        name=wire.market_name,
        banner_image_url=wire.banner_image_url or "",
        icon_url=wire.icon_url or "",
        featured_rank=wire.featured_rank,
        volume=wire.volume or "0",
        slug=wire.slug or "",
        status=_parse_status(wire.market_status),
        created_at=wire.created_at,
        activated_at=wire.activated_at,
        settled_at=wire.settled_at,
        winning_outcome=wire.winning_outcome,
        description=wire.description or "",
        definition=wire.definition or "",
        category=wire.category,
        tags=wire.tags,
        deposit_assets=deposit_assets,
        conditional_tokens=conditional_tokens,
        outcomes=outcomes,
        orderbook_pairs=orderbook_pairs,
        orderbook_ids=orderbook_ids,
        token_metadata=token_metadata,
    )
