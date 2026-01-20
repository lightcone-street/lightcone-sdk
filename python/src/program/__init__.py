"""On-chain program interaction module for Lightcone.

This module provides the client and utilities for interacting with
the Lightcone smart contract on Solana.
"""

from .accounts import (
    deserialize_exchange,
    deserialize_market,
    deserialize_order_status,
    deserialize_position,
    deserialize_user_nonce,
)
from .client import LightconePinocchioClient
from .ed25519 import (
    CrossRefEd25519Params,
    MatchIxOffsets,
    build_ed25519_batch_verify_instruction,
    build_ed25519_cross_ref_instruction,
    build_ed25519_verify_instruction,
    build_ed25519_verify_instruction_for_order,
    create_cross_ref_ed25519_instructions,
    create_single_cross_ref_ed25519_instruction,
)
from .instructions import (
    build_activate_market_instruction,
    build_add_deposit_mint_instruction,
    build_cancel_order_instruction,
    build_create_market_instruction,
    build_create_market_instruction_with_id,
    build_increment_nonce_instruction,
    build_initialize_instruction,
    build_match_orders_multi_instruction,
    build_merge_complete_set_instruction,
    build_mint_complete_set_instruction,
    build_redeem_winnings_instruction,
    build_set_operator_instruction,
    build_set_paused_instruction,
    build_settle_market_instruction,
    build_withdraw_from_position_instruction,
)
from .orders import (
    create_ask_order,
    create_bid_order,
    create_signed_ask_order,
    create_signed_bid_order,
    deserialize_compact_order,
    deserialize_full_order,
    hash_order,
    serialize_compact_order,
    serialize_full_order,
    sign_order,
    to_compact_order,
    validate_order,
    validate_signed_order,
    verify_order_signature,
)
from .pda import (
    get_all_conditional_mints,
    get_conditional_mint_pda,
    get_exchange_pda,
    get_market_pda,
    get_mint_authority_pda,
    get_order_status_pda,
    get_position_pda,
    get_user_nonce_pda,
    get_vault_pda,
)

__all__ = [
    # Client
    "LightconePinocchioClient",
    # Account Deserialization
    "deserialize_exchange",
    "deserialize_market",
    "deserialize_position",
    "deserialize_order_status",
    "deserialize_user_nonce",
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
]
