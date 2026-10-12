"""Wire types for committed custody funding (REST positions and the WS ``user`` channel).

The backend reads balances from the committed trading engine: every custody
token account a wallet can trade from is a *funding account* (a global deposit
account, or a conditional-token account of one market position). Amount
fields are ``Decimal`` in the account mint's own units unless the field name
ends in ``_atoms`` (raw signed integer atoms).
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from enum import Enum
from typing import Any

from ...shared.decimal_text import (
    DecimalText,
    wire_decimal_text,
    wire_opt_decimal_text,
)
from ...shared.types import CommitInfo, FundingSource, PubkeyStr
from ...shared.wire_decode import (
    take,
    wire_array,
    wire_bool,
    wire_commit_info,
    wire_decimal,
    wire_enum_or_unknown,
    wire_funding_source,
    wire_nonempty_str,
    wire_object,
    wire_opt_decimal,
    wire_opt_i128,
    wire_opt_str,
    wire_opt_u64,
    wire_pubkey,
    wire_str_or,
    wire_u64,
)


class ObservationState(str, Enum):
    """Evidence state of the last custody observation for a funding account."""

    #: The token account exists and ``raw_observed`` holds its balance.
    PRESENT = "present"
    #: The token account was observed not to exist.
    ABSENT = "absent"
    #: The account exists but is not a valid token account for this mint.
    INVALID = "invalid"
    #: No coherent observation has been accepted yet.
    UNOBSERVED = "unobserved"
    #: A state this SDK version does not know.
    UNKNOWN = "unknown"

    @classmethod
    def from_wire(cls, value: str) -> ObservationState:
        for member in cls:
            if member is not cls.UNKNOWN and member.value == value:
                return member
        return cls.UNKNOWN


def _opt_pubkey(value: Any, name: str) -> PubkeyStr | None:
    text = wire_nonempty_str(value, name)
    return None if text is None else PubkeyStr(text)


@dataclass(frozen=True, kw_only=True)
class FundingAccount:
    """One recorded funding account (REST positions, REST user orders, and the
    WS ``user`` snapshot).

    These are the committed quantities at the response's database snapshot,
    not live spending capacity: the engine alone decides whether an order can
    be funded.
    """

    #: Custody token account.
    account: PubkeyStr
    #: Token mint held by ``account`` (deposit mint or conditional mint).
    mint: PubkeyStr
    source: FundingSource
    #: Market of a conditional account; ``None`` for global accounts (the backend sends ``""``).
    market_pubkey: PubkeyStr | None
    #: Deposit mint that collateralizes ``mint``.
    deposit_mint: PubkeyStr
    #: Last observed token balance in ``mint`` units; ``None`` when unobserved.
    raw_observed: Decimal | None
    #: Amount reserved by open orders, in ``mint`` units (u128 aggregate).
    order_reserved: DecimalText
    #: Amount reserved by in-flight executions, in ``mint`` units (u128 aggregate).
    execution_reserved: DecimalText
    #: Recorded balance minus reservations, in raw signed atoms of ``mint``. Can
    #: be negative and is not spendable capacity; ``None`` when unobserved.
    signed_remaining_atoms: int | None
    #: Recorded observation boundary (opaque).
    accepted_boundary: str
    #: Solana slot of the last accepted observation.
    observed_slot: int | None
    #: Blockhash of the last accepted observation (empty when unobserved).
    observed_blockhash: str
    observation_state: ObservationState

    @staticmethod
    def from_dict(value: Any, name: str = "funding_account") -> FundingAccount:
        data = wire_object(value, name)
        return FundingAccount(
            account=wire_pubkey(take(data, "account"), f"{name}.account"),
            mint=wire_pubkey(take(data, "mint"), f"{name}.mint"),
            source=wire_funding_source(take(data, "source"), f"{name}.source"),
            market_pubkey=_opt_pubkey(
                take(data, "market_pubkey"), f"{name}.market_pubkey"
            ),
            deposit_mint=wire_pubkey(
                take(data, "deposit_mint"), f"{name}.deposit_mint"
            ),
            raw_observed=wire_opt_decimal(
                take(data, "raw_observed"), f"{name}.raw_observed"
            ),
            order_reserved=wire_decimal_text(
                take(data, "order_reserved"), f"{name}.order_reserved"
            ),
            execution_reserved=wire_decimal_text(
                take(data, "execution_reserved"), f"{name}.execution_reserved"
            ),
            signed_remaining_atoms=wire_opt_i128(
                take(data, "signed_remaining_atoms"), f"{name}.signed_remaining_atoms"
            ),
            accepted_boundary=wire_str_or(
                take(data, "accepted_boundary"), f"{name}.accepted_boundary", ""
            ),
            observed_slot=wire_opt_u64(
                take(data, "observed_slot"), f"{name}.observed_slot"
            ),
            observed_blockhash=wire_str_or(
                take(data, "observed_blockhash"), f"{name}.observed_blockhash", ""
            ),
            observation_state=wire_enum_or_unknown(
                take(data, "observation_state"),
                f"{name}.observation_state",
                ObservationState,
                ObservationState.UNKNOWN,
            ),
        )


@dataclass(frozen=True, kw_only=True)
class FundingUpdate:
    """Live funding-account update on the WS ``user`` channel (``event_type: "funding"``).

    Unlike :class:`FundingAccount` this carries the engine's transient
    readiness view: ``available`` and ``ready`` describe spending capacity at
    publication time and are forced to zero/``False`` when the fact is not
    actionable.
    """

    commit: CommitInfo
    user_pubkey: PubkeyStr
    account: PubkeyStr
    mint: PubkeyStr
    source: FundingSource
    #: Market of a conditional account; ``None`` for global accounts.
    market_pubkey: PubkeyStr | None
    #: Last observed token balance in ``mint`` units.
    raw_observed: Decimal | None
    #: Observed balance usable for trading, in ``mint`` units.
    usable_observed: Decimal | None
    #: Amount reserved by open orders, in ``mint`` units.
    order_reserved: DecimalText
    #: Amount reserved by in-flight executions, in ``mint`` units.
    execution_reserved: DecimalText
    #: Usable balance minus reservations, in ``mint`` units; may be negative.
    signed_remaining: DecimalText | None
    #: Spendable amount for new orders, in ``mint`` units.
    available: Decimal
    #: Whether the engine will fund new orders from this account.
    ready: bool
    accepted_boundary: str | None
    #: Solana slot of the observation.
    slot: int | None
    blockhash: str | None

    @staticmethod
    def from_dict(value: Any, name: str = "funding") -> FundingUpdate:
        """Decode a live ``funding`` fact (the ``event_type`` tag is ignored)."""
        data = wire_object(value, name)
        return FundingUpdate(
            commit=wire_commit_info(data, name),
            user_pubkey=wire_pubkey(take(data, "user_pubkey"), f"{name}.user_pubkey"),
            account=wire_pubkey(take(data, "account"), f"{name}.account"),
            mint=wire_pubkey(take(data, "mint"), f"{name}.mint"),
            source=wire_funding_source(take(data, "source"), f"{name}.source"),
            market_pubkey=_opt_pubkey(
                take(data, "market_pubkey"), f"{name}.market_pubkey"
            ),
            raw_observed=wire_opt_decimal(
                take(data, "raw_observed"), f"{name}.raw_observed"
            ),
            usable_observed=wire_opt_decimal(
                take(data, "usable_observed"), f"{name}.usable_observed"
            ),
            order_reserved=wire_decimal_text(
                take(data, "order_reserved"), f"{name}.order_reserved"
            ),
            execution_reserved=wire_decimal_text(
                take(data, "execution_reserved"), f"{name}.execution_reserved"
            ),
            signed_remaining=wire_opt_decimal_text(
                take(data, "signed_remaining"), f"{name}.signed_remaining"
            ),
            available=wire_decimal(take(data, "available"), f"{name}.available"),
            ready=wire_bool(take(data, "ready"), f"{name}.ready"),
            accepted_boundary=wire_opt_str(
                take(data, "accepted_boundary"), f"{name}.accepted_boundary"
            ),
            slot=wire_opt_u64(take(data, "slot"), f"{name}.slot"),
            blockhash=wire_opt_str(take(data, "blockhash"), f"{name}.blockhash"),
        )


@dataclass(frozen=True, kw_only=True)
class PositionsResponse:
    """Response for all four positions routes: ``GET /api/users/positions``,
    ``/api/users/{user}/positions``, ``/api/users/markets/{market}/positions`` and
    ``/api/users/{user}/markets/{market}/positions``.

    Market-scoped routes keep global accounts and the conditional accounts of
    that market. Pages are filtered after a bounded scan, so a page can be
    empty while ``has_more`` is true.
    """

    owner: PubkeyStr
    #: Market filter of a market-scoped route; ``None`` for wallet-wide routes.
    market_pubkey: PubkeyStr | None
    funding_accounts: list[FundingAccount] = field(default_factory=list)
    #: Database revision of this page; later pages may be newer.
    committed_revision: int
    projection_generation: int
    has_more: bool
    #: Funding-account cursor for the next page (authenticated routes only; the
    #: public ``{user}`` routes reject every query string).
    next_cursor: str | None

    @staticmethod
    def from_dict(value: Any) -> PositionsResponse:
        """Decode a positions page (all four positions routes)."""
        data = wire_object(value, "positions")
        market = wire_opt_str(take(data, "market_pubkey"), "positions.market_pubkey")
        return PositionsResponse(
            owner=wire_pubkey(take(data, "owner"), "positions.owner"),
            market_pubkey=None if market is None else PubkeyStr(market),
            funding_accounts=wire_array(
                take(data, "funding_accounts"),
                "positions.funding_accounts",
                FundingAccount.from_dict,
            ),
            committed_revision=wire_u64(
                take(data, "committed_revision"), "positions.committed_revision"
            ),
            projection_generation=wire_u64(
                take(data, "projection_generation"), "positions.projection_generation"
            ),
            has_more=wire_bool(take(data, "has_more"), "positions.has_more"),
            next_cursor=wire_opt_str(
                take(data, "next_cursor"), "positions.next_cursor"
            ),
        )


#: Market-scoped positions share the wallet-wide response shape.
MarketPositionsResponse = PositionsResponse


__all__ = [
    "FundingAccount",
    "FundingUpdate",
    "MarketPositionsResponse",
    "ObservationState",
    "PositionsResponse",
]
