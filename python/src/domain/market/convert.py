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
    return Status.from_str(s)


def market_from_wire(wire: MarketWire) -> Market:
    """Convert a MarketWire to a Market domain type."""
    outcomes = [
        Outcome(
            index=o.get("index", i),
            name=o.get("name", ""),
            icon_url=o.get("icon_url", ""),
        )
        for i, o in enumerate(wire.outcomes)
    ]

    # Build conditional tokens and token_metadata from deposit_assets wire
    conditional_tokens: list[ConditionalToken] = []
    token_metadata: dict[str, TokenMetadata] = {}
    deposit_assets: list[DepositAsset] = []
    orderbook_ids: list[str] = []

    for da in wire.deposit_assets:
        deposit_asset = DepositAsset(
            id=da.get("id", 0),
            market_pda=da.get("market_pubkey", ""),
            deposit_asset=da.get("deposit_asset", ""),
            num_outcomes=da.get("num_outcomes", 0),
            name=da.get("display_name", ""),
            symbol=da.get("token_symbol", da.get("symbol", "")),
            description=da.get("description"),
            decimals=da.get("decimals") or 6,
            icon_url=da.get("icon_url", ""),
        )
        deposit_assets.append(deposit_asset)

        for ct in da.get("conditional_mints", []):
            conditional_tokens.append(ConditionalToken(
                mint=ct.get("token_address", ""),
                outcome_index=ct.get("outcome_index", 0),
                outcome=ct.get("outcome", ""),
                deposit_asset=da.get("deposit_asset", ""),
                deposit_symbol=da.get("token_symbol", da.get("symbol", "")),
                name=ct.get("display_name", ct.get("name", "")),
                symbol=ct.get("short_name", ct.get("symbol", "")),
                description=ct.get("description"),
                decimals=ct.get("decimals") or 6,
                icon_url=ct.get("icon_url", ""),
            ))
            mint = ct.get("token_address", "")
            if mint:
                token_metadata[mint] = TokenMetadata(
                    pubkey=mint,
                    symbol=ct.get("short_name", ct.get("symbol", "")),
                    decimals=ct.get("decimals") or 6,
                    icon_url=ct.get("icon_url", ""),
                    name=ct.get("display_name", ct.get("name", "")),
                )

    orderbook_pairs = []
    for ob in wire.orderbooks:
        ob_id = ob.get("orderbook_id", "")
        orderbook_pairs.append(OrderBookPairSummary(
            id=ob.get("id", 0),
            market_pubkey=ob.get("market_pubkey", wire.market_pubkey),
            orderbook_id=ob_id,
            base_token=ob.get("base_token", ""),
            quote_token=ob.get("quote_token", ""),
            outcome_index=ob.get("outcome_index", 0),
            tick_size=ob.get("tick_size", 0),
            total_bids=ob.get("total_bids", 0),
            total_asks=ob.get("total_asks", 0),
            last_trade_price=ob.get("last_trade_price"),
            last_trade_time=ob.get("last_trade_time"),
            active=ob.get("active", True),
        ))
        if ob_id:
            orderbook_ids.append(ob_id)

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
