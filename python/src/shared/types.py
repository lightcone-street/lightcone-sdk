"""Type definitions for the Lightcone SDK."""

from dataclasses import dataclass, field
from enum import IntEnum
from typing import Optional

from solders.pubkey import Pubkey


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


@dataclass
class Market:
    """Market account data."""

    market_id: int
    num_outcomes: int
    status: MarketStatus
    winning_outcome: Optional[int]
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
class FullOrder:
    """Full order structure with all fields including signature."""

    nonce: int
    maker: Pubkey
    market: Pubkey
    base_mint: Pubkey
    quote_mint: Pubkey
    side: OrderSide
    maker_amount: int
    taker_amount: int
    expiration: int
    signature: bytes = field(default_factory=lambda: bytes(64))


@dataclass
class CompactOrder:
    """Compact order structure for instruction data (excludes market/mints)."""

    nonce: int
    maker: Pubkey
    side: OrderSide
    maker_amount: int
    taker_amount: int
    expiration: int


@dataclass
class OutcomeMetadata:
    """Metadata for a market outcome."""

    name: str
    symbol: str
    uri: str


@dataclass
class MakerFill:
    """Maker order with fill amount for matching."""

    order: FullOrder
    fill_amount: int


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

    payer: Pubkey
    market: Pubkey
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
    market: Pubkey
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
    position: Pubkey
    mint: Pubkey
    amount: int


@dataclass
class ActivateMarketParams:
    """Parameters for activating a market."""

    authority: Pubkey
    market: Pubkey


@dataclass
class MatchOrdersMultiParams:
    """Parameters for matching multiple orders."""

    operator: Pubkey
    market: Pubkey
    base_mint: Pubkey
    quote_mint: Pubkey
    taker_order: FullOrder
    maker_fills: list[MakerFill]


@dataclass
class BidOrderParams:
    """Parameters for creating a bid order."""

    nonce: int
    maker: Pubkey
    market: Pubkey
    base_mint: Pubkey
    quote_mint: Pubkey
    maker_amount: int  # Quote tokens given
    taker_amount: int  # Base tokens received
    expiration: int


@dataclass
class AskOrderParams:
    """Parameters for creating an ask order."""

    nonce: int
    maker: Pubkey
    market: Pubkey
    base_mint: Pubkey
    quote_mint: Pubkey
    maker_amount: int  # Base tokens given
    taker_amount: int  # Quote tokens received
    expiration: int


@dataclass
class BuildResult:
    """Result of building a transaction."""

    transaction: "Transaction"  # Lazy import to avoid circular deps
    signers: list[Pubkey]


class Resolution(IntEnum):
    """Price history candle resolution."""

    ONE_MINUTE = 0
    FIVE_MINUTES = 1
    FIFTEEN_MINUTES = 2
    ONE_HOUR = 3
    FOUR_HOURS = 4
    ONE_DAY = 5

    def as_str(self) -> str:
        """Get the string representation for API calls."""
        mapping = {
            Resolution.ONE_MINUTE: "1m",
            Resolution.FIVE_MINUTES: "5m",
            Resolution.FIFTEEN_MINUTES: "15m",
            Resolution.ONE_HOUR: "1h",
            Resolution.FOUR_HOURS: "4h",
            Resolution.ONE_DAY: "1d",
        }
        return mapping[self]

    @classmethod
    def from_str(cls, s: str) -> "Resolution":
        """Parse a resolution string."""
        mapping = {
            "1m": cls.ONE_MINUTE,
            "5m": cls.FIVE_MINUTES,
            "15m": cls.FIFTEEN_MINUTES,
            "1h": cls.ONE_HOUR,
            "4h": cls.FOUR_HOURS,
            "1d": cls.ONE_DAY,
        }
        if s not in mapping:
            raise ValueError(f"Invalid resolution: {s}")
        return mapping[s]

    def __str__(self) -> str:
        return self.as_str()
