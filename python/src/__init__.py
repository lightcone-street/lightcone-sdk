"""Lightcone SDK - Python SDK for the Lightcone protocol on Solana.

This SDK provides three main modules:
- `program`: On-chain program interaction (smart contract)
- `api`: REST API client (coming soon)
- `websocket`: Real-time data streaming (coming soon)

Example:
    from lightcone_sdk import LightconePinocchioClient, PROGRAM_ID

    # Or import from specific modules
    from lightcone_sdk.program import LightconePinocchioClient
    from lightcone_sdk.shared import PROGRAM_ID
"""

__version__ = "0.1.0"

# ============================================================================
# MODULE IMPORTS
# ============================================================================

# Import submodules for namespace access
from . import api
from . import program
from . import shared
from . import websocket

# ============================================================================
# CONVENIENCE RE-EXPORTS FROM PROGRAM MODULE
# ============================================================================

from .program import (
    LightconePinocchioClient,
    # Account Deserialization
    deserialize_exchange,
    deserialize_market,
    deserialize_order_status,
    deserialize_position,
    deserialize_user_nonce,
    # PDA Functions
    get_exchange_pda,
    get_market_pda,
    get_vault_pda,
    get_mint_authority_pda,
    get_conditional_mint_pda,
    get_order_status_pda,
    get_user_nonce_pda,
    get_position_pda,
    get_all_conditional_mints,
    # Order Functions
    create_bid_order,
    create_ask_order,
    create_signed_bid_order,
    create_signed_ask_order,
    hash_order,
    sign_order,
    verify_order_signature,
    serialize_full_order,
    deserialize_full_order,
    serialize_compact_order,
    deserialize_compact_order,
    to_compact_order,
    validate_order,
    validate_signed_order,
    # Ed25519 Functions
    build_ed25519_verify_instruction,
    build_ed25519_verify_instruction_for_order,
    build_ed25519_batch_verify_instruction,
    build_ed25519_cross_ref_instruction,
    create_cross_ref_ed25519_instructions,
    create_single_cross_ref_ed25519_instruction,
    CrossRefEd25519Params,
    MatchIxOffsets,
    # Instruction Builders
    build_initialize_instruction,
    build_create_market_instruction,
    build_create_market_instruction_with_id,
    build_add_deposit_mint_instruction,
    build_mint_complete_set_instruction,
    build_merge_complete_set_instruction,
    build_cancel_order_instruction,
    build_increment_nonce_instruction,
    build_settle_market_instruction,
    build_redeem_winnings_instruction,
    build_set_paused_instruction,
    build_set_operator_instruction,
    build_withdraw_from_position_instruction,
    build_activate_market_instruction,
    build_match_orders_multi_instruction,
)

# ============================================================================
# CONVENIENCE RE-EXPORTS FROM SHARED MODULE
# ============================================================================

from .shared import (
    # Constants
    PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    SYSTEM_PROGRAM_ID,
    RENT_SYSVAR_ID,
    INSTRUCTIONS_SYSVAR_ID,
    ED25519_PROGRAM_ID,
    MAX_OUTCOMES,
    MIN_OUTCOMES,
    MAX_MAKERS,
    # Types - Enums
    MarketStatus,
    OrderSide,
    # Types - Account Data
    Exchange,
    Market,
    Position,
    OrderStatus,
    UserNonce,
    # Types - Orders
    FullOrder,
    CompactOrder,
    OutcomeMetadata,
    MakerFill,
    # Types - Params
    InitializeParams,
    CreateMarketParams,
    AddDepositMintParams,
    MintCompleteSetParams,
    MergeCompleteSetParams,
    SettleMarketParams,
    RedeemWinningsParams,
    WithdrawFromPositionParams,
    ActivateMarketParams,
    MatchOrdersMultiParams,
    BidOrderParams,
    AskOrderParams,
    BuildResult,
    # Errors
    LightconeError,
    InvalidDiscriminatorError,
    AccountNotFoundError,
    InvalidAccountDataError,
    InvalidOrderError,
    InvalidSignatureError,
    OrderExpiredError,
    InsufficientBalanceError,
    MarketNotActiveError,
    ExchangePausedError,
    InvalidOutcomeError,
    TooManyMakersError,
    OrdersDoNotCrossError,
    # Utils
    keccak256,
    derive_condition_id,
    get_associated_token_address,
    get_associated_token_address_2022,
    orders_cross,
)

__all__ = [
    # Version
    "__version__",
    # Modules
    "program",
    "shared",
    "api",
    "websocket",
    # Client
    "LightconePinocchioClient",
    # Types - Enums
    "MarketStatus",
    "OrderSide",
    # Types - Account Data
    "Exchange",
    "Market",
    "Position",
    "OrderStatus",
    "UserNonce",
    # Types - Orders
    "FullOrder",
    "CompactOrder",
    "OutcomeMetadata",
    "MakerFill",
    # Types - Params
    "InitializeParams",
    "CreateMarketParams",
    "AddDepositMintParams",
    "MintCompleteSetParams",
    "MergeCompleteSetParams",
    "SettleMarketParams",
    "RedeemWinningsParams",
    "WithdrawFromPositionParams",
    "ActivateMarketParams",
    "MatchOrdersMultiParams",
    "BidOrderParams",
    "AskOrderParams",
    "BuildResult",
    # Errors
    "LightconeError",
    "InvalidDiscriminatorError",
    "AccountNotFoundError",
    "InvalidAccountDataError",
    "InvalidOrderError",
    "InvalidSignatureError",
    "OrderExpiredError",
    "InsufficientBalanceError",
    "MarketNotActiveError",
    "ExchangePausedError",
    "InvalidOutcomeError",
    "TooManyMakersError",
    "OrdersDoNotCrossError",
    # Constants
    "PROGRAM_ID",
    "TOKEN_PROGRAM_ID",
    "TOKEN_2022_PROGRAM_ID",
    "ASSOCIATED_TOKEN_PROGRAM_ID",
    "SYSTEM_PROGRAM_ID",
    "RENT_SYSVAR_ID",
    "INSTRUCTIONS_SYSVAR_ID",
    "ED25519_PROGRAM_ID",
    "MAX_OUTCOMES",
    "MIN_OUTCOMES",
    "MAX_MAKERS",
    # PDA Functions
    "get_exchange_pda",
    "get_market_pda",
    "get_vault_pda",
    "get_mint_authority_pda",
    "get_conditional_mint_pda",
    "get_order_status_pda",
    "get_user_nonce_pda",
    "get_position_pda",
    "get_all_conditional_mints",
    # Account Deserialization
    "deserialize_exchange",
    "deserialize_market",
    "deserialize_position",
    "deserialize_order_status",
    "deserialize_user_nonce",
    # Order Functions
    "create_bid_order",
    "create_ask_order",
    "create_signed_bid_order",
    "create_signed_ask_order",
    "hash_order",
    "sign_order",
    "verify_order_signature",
    "serialize_full_order",
    "deserialize_full_order",
    "serialize_compact_order",
    "deserialize_compact_order",
    "to_compact_order",
    "validate_order",
    "validate_signed_order",
    # Ed25519 Functions
    "build_ed25519_verify_instruction",
    "build_ed25519_verify_instruction_for_order",
    "build_ed25519_batch_verify_instruction",
    "build_ed25519_cross_ref_instruction",
    "create_cross_ref_ed25519_instructions",
    "create_single_cross_ref_ed25519_instruction",
    "CrossRefEd25519Params",
    "MatchIxOffsets",
    # Instruction Builders
    "build_initialize_instruction",
    "build_create_market_instruction",
    "build_create_market_instruction_with_id",
    "build_add_deposit_mint_instruction",
    "build_mint_complete_set_instruction",
    "build_merge_complete_set_instruction",
    "build_cancel_order_instruction",
    "build_increment_nonce_instruction",
    "build_settle_market_instruction",
    "build_redeem_winnings_instruction",
    "build_set_paused_instruction",
    "build_set_operator_instruction",
    "build_withdraw_from_position_instruction",
    "build_activate_market_instruction",
    "build_match_orders_multi_instruction",
    # Utility Functions
    "keccak256",
    "derive_condition_id",
    "get_associated_token_address",
    "get_associated_token_address_2022",
    "orders_cross",
]
