"""Type definitions for the Lightcone program module."""

from dataclasses import dataclass, field
from enum import IntEnum
from typing import Optional

from solders.pubkey import Pubkey
from solders.transaction import Transaction

from ..shared.types import DepositSource


class MarketStatus(IntEnum):
    """Status of a market."""

    PENDING = 0
    ACTIVE = 1
    RESOLVED = 2
    CANCELLED = 3


class OrderSide(IntEnum):
    """Side of an order in the orderbook."""

    BID = 0  # Buyer wants base tokens, gives quote tokens
    ASK = 1  # Seller offers base tokens, receives quote tokens


@dataclass
class Exchange:
    """Exchange account data."""

    authority: Pubkey
    operator: Pubkey
    market_count: int
    paused: bool
    bump: int
    deposit_token_count: int = 0


@dataclass
class Market:
    """Market account data."""

    market_id: int
    num_outcomes: int
    status: MarketStatus
    winning_outcome: int  # u8 raw value (255 = no winner)
    has_winning_outcome: bool
    bump: int
    oracle: Pubkey
    question_id: bytes
    condition_id: bytes


@dataclass
class Position:
    """Position account data."""

    owner: Pubkey
    market: Pubkey
    bump: int


@dataclass
class OrderStatus:
    """Order status account data."""

    remaining: int
    is_cancelled: bool


@dataclass
class UserNonce:
    """User nonce account data."""

    nonce: int


@dataclass
class Orderbook:
    """Orderbook account data."""

    market: Pubkey
    mint_a: Pubkey
    mint_b: Pubkey
    lookup_table: Pubkey
    base_index: int
    bump: int


@dataclass
class SignedOrder:
    """Signed order structure with all fields including signature (233 bytes).

    Note: nonce is u32 range (0 to 2^32-1) but serialized as u64 LE on wire for compatibility.
    """

    nonce: int  # u32 range
    maker: Pubkey
    market: Pubkey
    base_mint: Pubkey
    quote_mint: Pubkey
    side: OrderSide
    amount_in: int
    amount_out: int
    expiration: int
    salt: int = 0  # u64
    signature: bytes = field(default_factory=lambda: bytes(64))


# Backward compatibility alias
FullOrder = SignedOrder


@dataclass
class Order:
    """Compact order structure for instruction data (37 bytes, no maker field).

    Layout: [0..4] nonce(u32) | [4..12] salt(u64) | [12] side(u8) |
            [13..21] amount_in(u64) | [21..29] amount_out(u64) | [29..37] expiration(i64)
    """

    nonce: int
    side: OrderSide
    amount_in: int
    amount_out: int
    expiration: int
    salt: int = 0  # u64


# Backward compatibility alias
CompactOrder = Order


@dataclass
class OutcomeMetadata:
    """Metadata for a market outcome."""

    name: str
    symbol: str
    uri: str


@dataclass
class MakerFill:
    """Per-maker fill info for deposit_and_swap."""

    order: SignedOrder
    maker_fill_amount: int
    taker_fill_amount: int
    deposit_mint: Pubkey
    is_full_fill: bool = False
    is_deposit: bool = False


# Parameter types for client methods


@dataclass
class InitializeParams:
    """Parameters for initializing the exchange."""

    authority: Pubkey


@dataclass
class CreateMarketParams:
    """Parameters for creating a new market."""

    authority: Pubkey
    num_outcomes: int
    oracle: Pubkey
    question_id: bytes


@dataclass
class AddDepositMintParams:
    """Parameters for adding a deposit mint to a market."""

    authority: Pubkey
    deposit_mint: Pubkey
    outcome_metadata: list[OutcomeMetadata]


@dataclass
class MintCompleteSetParams:
    """Parameters for minting a complete set of outcome tokens."""

    user: Pubkey
    market: Pubkey
    deposit_mint: Pubkey
    amount: int


@dataclass
class MergeCompleteSetParams:
    """Parameters for merging a complete set back to collateral."""

    user: Pubkey
    market: Pubkey
    deposit_mint: Pubkey
    amount: int


@dataclass
class SettleMarketParams:
    """Parameters for settling a market."""

    oracle: Pubkey
    market_id: int
    winning_outcome: int


@dataclass
class RedeemWinningsParams:
    """Parameters for redeeming winning tokens."""

    user: Pubkey
    market: Pubkey
    deposit_mint: Pubkey
    amount: int


@dataclass
class WithdrawFromPositionParams:
    """Parameters for withdrawing tokens from a position."""

    user: Pubkey
    market: Pubkey
    mint: Pubkey
    amount: int
    outcome_index: int


@dataclass
class ActivateMarketParams:
    """Parameters for activating a market."""

    authority: Pubkey
    market_id: int


@dataclass
class MatchOrdersMultiParams:
    """Parameters for matching multiple orders."""

    operator: Pubkey
    market: Pubkey
    base_mint: Pubkey
    quote_mint: Pubkey
    taker_order: SignedOrder
    maker_orders: list[SignedOrder]
    maker_fill_amounts: list[int]
    taker_fill_amounts: list[int]
    full_fill_bitmask: int


@dataclass
class CreateOrderbookParams:
    """Parameters for creating an orderbook."""

    authority: Pubkey
    market: Pubkey
    mint_a: Pubkey
    mint_b: Pubkey
    recent_slot: int
    base_index: int


@dataclass
class SetAuthorityParams:
    """Parameters for setting a new authority."""

    current_authority: Pubkey
    new_authority: Pubkey


@dataclass
class BidOrderParams:
    """Parameters for creating a bid order."""

    nonce: int  # u32 range
    maker: Pubkey
    market: Pubkey
    base_mint: Pubkey
    quote_mint: Pubkey
    amount_in: int  # Quote tokens given
    amount_out: int  # Base tokens received
    expiration: int
    salt: int = 0  # u64


@dataclass
class AskOrderParams:
    """Parameters for creating an ask order."""

    nonce: int  # u32 range
    maker: Pubkey
    market: Pubkey
    base_mint: Pubkey
    quote_mint: Pubkey
    amount_in: int  # Base tokens given
    amount_out: int  # Quote tokens received
    expiration: int
    salt: int = 0  # u64


@dataclass
class GlobalDepositToken:
    """Global deposit token account data (48 bytes)."""

    mint: Pubkey
    active: bool
    bump: int
    index: int  # u16


@dataclass
class WhitelistDepositTokenParams:
    """Parameters for whitelisting a deposit token for global deposits."""

    authority: Pubkey
    mint: Pubkey


@dataclass
class DepositToGlobalParams:
    """Parameters for depositing tokens to a global deposit account."""

    user: Pubkey
    mint: Pubkey
    amount: int


@dataclass
class GlobalToMarketDepositParams:
    """Parameters for transferring from global deposit to a market vault."""

    user: Pubkey
    market: Pubkey
    deposit_mint: Pubkey
    amount: int


@dataclass
class InitPositionTokensParams:
    """Parameters for initializing position token accounts and ALT."""

    payer: Pubkey
    user: Pubkey
    market: Pubkey
    deposit_mints: list[Pubkey]
    recent_slot: int


@dataclass
class DepositAndSwapParams:
    """Parameters for deposit-and-swap (atomic deposit + mint + swap)."""

    operator: Pubkey
    market: Pubkey
    base_mint: Pubkey
    quote_mint: Pubkey
    taker_order: SignedOrder
    taker_is_full_fill: bool
    taker_is_deposit: bool
    taker_deposit_mint: Pubkey
    num_outcomes: int
    makers: list[MakerFill]


@dataclass
class ExtendPositionTokensParams:
    """Parameters for extending a position ALT with new deposit mints."""

    payer: Pubkey
    user: Pubkey
    market: Pubkey
    lookup_table: Pubkey
    deposit_mints: list[Pubkey]


@dataclass
class WithdrawFromGlobalParams:
    """Parameters for withdrawing tokens from a global deposit account."""

    user: Pubkey
    mint: Pubkey
    amount: int


@dataclass
class BuildResult:
    """Result of building a transaction."""

    transaction: Transaction
    signers: list[Pubkey]


# ============================================================================
# Unified Deposit/Withdraw Parameters
# ============================================================================


@dataclass
class DepositParams:
    """Unified deposit parameters — dispatches to global or market deposit
    based on the client's deposit source setting.

    Prefer using the builder via ``client.positions().deposit()`` which
    pre-seeds the client's deposit source. Direct construction is also available.
    """

    user: Pubkey
    mint: Pubkey
    amount: int
    market: object = None  # Optional domain Market (has .pubkey str, .outcomes list)
    deposit_source: Optional[DepositSource] = None


@dataclass
class MarketWithdrawContext:
    """Market-specific context required for withdrawals when deposit source is Market."""

    market: object  # domain Market (has .pubkey str, .outcomes list)
    outcome_index: int
    is_token_2022: bool = False


@dataclass
class WithdrawParams:
    """Unified withdraw parameters — dispatches to global or market withdrawal
    based on the client's deposit source setting.

    Prefer using the builder via ``client.positions().withdraw()`` which
    pre-seeds the client's deposit source. Direct construction is also available.
    """

    user: Pubkey
    mint: Pubkey
    amount: int
    market_context: Optional[MarketWithdrawContext] = None
    deposit_source: Optional[DepositSource] = None


# Aliases matching Rust SDK naming (PR #46)
BuildDepositParams = MintCompleteSetParams
BuildMergeParams = MergeCompleteSetParams
