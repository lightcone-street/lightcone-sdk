"""Market wire-to-domain conversion."""

from typing import Optional
from . import (
    Market, MarketValidationError, Status, Outcome, ConditionalToken, DepositAsset,
    DepositAssetPair, GlobalDepositAsset, TokenMetadata, token_display_priority,
)
from ...error import SdkError
from ..orderbook import OrderBookPair
from .wire import GlobalDepositAssetWire, MarketWire


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
            ct_name = ct.outcome
            ct_symbol = ct.short_symbol or ct.symbol
            conditional_tokens.append(ConditionalToken(
                pubkey=ct.token_address,
                outcome_index=ct.outcome_index,
                id=ct.id,
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

    # Build a lookup from pubkey to ConditionalToken for orderbook pairs.
    ct_by_pubkey: dict[str, ConditionalToken] = {ct.pubkey: ct for ct in conditional_tokens}

    orderbook_pairs = []
    for ob in wire.orderbooks:
        base_ct = ct_by_pubkey.get(ob.base_token, ConditionalToken(pubkey=ob.base_token, outcome_index=0))
        quote_ct = ct_by_pubkey.get(ob.quote_token, ConditionalToken(pubkey=ob.quote_token, outcome_index=0))
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

    deposit_asset_pairs = _derive_deposit_asset_pairs(deposit_assets, orderbook_pairs)
    deposit_asset_pairs.sort(
        key=lambda pair: (token_display_priority(pair.base), pair.base.symbol)
    )

    if not deposit_asset_pairs:
        identifier = wire.market_pubkey or str(wire.market_id)
        raise MarketValidationError(
            f"Market validation errors ({identifier}): Missing deposit asset pairs",
            ["Missing deposit asset pairs"],
        )

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
        deposit_asset_pairs=deposit_asset_pairs,
        conditional_tokens=conditional_tokens,
        outcomes=outcomes,
        orderbook_pairs=orderbook_pairs,
        orderbook_ids=orderbook_ids,
        token_metadata=token_metadata,
    )


def _derive_deposit_asset_pairs(
    deposit_assets: list[DepositAsset],
    orderbook_pairs: list[OrderBookPair],
) -> list[DepositAssetPair]:
    """Derive unique base/quote deposit-asset pairs.

    Deduplicated by ``(base_pubkey, quote_pubkey)``; orderbook pairs whose
    base or quote deposit asset is not present in ``deposit_assets`` are
    skipped. Order is not guaranteed.
    """
    seen: dict[tuple[str, str], DepositAssetPair] = {}
    for pair in orderbook_pairs:
        base = next(
            (a for a in deposit_assets if a.deposit_asset == pair.base.deposit_asset),
            None,
        )
        quote = next(
            (a for a in deposit_assets if a.deposit_asset == pair.quote.deposit_asset),
            None,
        )
        if base is None or quote is None:
            continue
        key = (base.deposit_asset, quote.deposit_asset)
        if key not in seen:
            seen[key] = DepositAssetPair(
                id=f"{base.deposit_asset}-{quote.deposit_asset}",
                base=base,
                quote=quote,
            )
    return list(seen.values())


def global_deposit_asset_from_wire(
    wire: GlobalDepositAssetWire,
) -> GlobalDepositAsset:
    """Convert a ``GlobalDepositAssetWire`` to a ``GlobalDepositAsset``.

    Raises ``SdkError`` with a rendered multi-error message when required
    fields (``display_name``, ``symbol``, ``icon_url``, ``decimals``) are
    missing on the wire payload.
    """
    errors: list[str] = []

    if wire.display_name is None:
        errors.append(f"Missing display name: {wire.mint}")
    if wire.symbol is None:
        errors.append(f"Missing symbol: {wire.mint}")
    if wire.icon_url is None:
        errors.append(f"Missing icon URL: {wire.mint}")
    if wire.decimals is None:
        errors.append(f"Missing decimals: {wire.mint}")

    if errors:
        rendered = "\n".join(f"  - {error}" for error in errors)
        raise SdkError(
            f"Token validation errors ({wire.mint}):\n{rendered}"
        )

    return GlobalDepositAsset(
        id=wire.id,
        deposit_asset=wire.mint,
        name=wire.display_name or "",
        symbol=wire.symbol or "",
        description=wire.description,
        decimals=wire.decimals or 0,
        icon_url=wire.icon_url or "",
        whitelist_index=wire.whitelist_index,
        active=wire.active,
    )
