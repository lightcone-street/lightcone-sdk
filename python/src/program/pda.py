"""PDA (Program Derived Address) derivation functions for the Lightcone SDK."""

from typing import Tuple

from solders.pubkey import Pubkey

from .constants import (
    ALT_PROGRAM_ID,
    ORDERBOOK_SEED,
    PROGRAM_ID,
    SEED_CENTRAL_STATE,
    SEED_CONDITIONAL_MINT,
    SEED_MARKET,
    SEED_MINT_AUTHORITY,
    SEED_ORDER_STATUS,
    SEED_POSITION,
    SEED_USER_NONCE,
    SEED_VAULT,
)
from .utils import encode_u64, encode_u8


def get_exchange_pda(program_id: Pubkey = PROGRAM_ID) -> Tuple[Pubkey, int]:
    """Derive the exchange PDA.

    Seeds: ["central_state"]
    """
    return Pubkey.find_program_address([SEED_CENTRAL_STATE], program_id)


def get_market_pda(
    market_id: int,
    program_id: Pubkey = PROGRAM_ID,
) -> Tuple[Pubkey, int]:
    """Derive the market PDA for a given market ID.

    Seeds: ["market", market_id (u64 LE)]
    """
    return Pubkey.find_program_address(
        [SEED_MARKET, encode_u64(market_id)],
        program_id,
    )


def get_vault_pda(
    deposit_mint: Pubkey,
    market: Pubkey,
    program_id: Pubkey = PROGRAM_ID,
) -> Tuple[Pubkey, int]:
    """Derive the vault PDA for a deposit mint in a market.

    Seeds: ["market_deposit_token_account", deposit_mint, market]
    """
    return Pubkey.find_program_address(
        [SEED_VAULT, bytes(deposit_mint), bytes(market)],
        program_id,
    )


def get_mint_authority_pda(
    market: Pubkey,
    program_id: Pubkey = PROGRAM_ID,
) -> Tuple[Pubkey, int]:
    """Derive the mint authority PDA for a market.

    Seeds: ["market_mint_authority", market]
    """
    return Pubkey.find_program_address(
        [SEED_MINT_AUTHORITY, bytes(market)],
        program_id,
    )


def get_conditional_mint_pda(
    market: Pubkey,
    deposit_mint: Pubkey,
    outcome_index: int,
    program_id: Pubkey = PROGRAM_ID,
) -> Tuple[Pubkey, int]:
    """Derive the conditional mint PDA for a specific outcome.

    Seeds: ["conditional_mint", market, deposit_mint, outcome_index (u8)]
    """
    return Pubkey.find_program_address(
        [
            SEED_CONDITIONAL_MINT,
            bytes(market),
            bytes(deposit_mint),
            encode_u8(outcome_index),
        ],
        program_id,
    )


def get_order_status_pda(
    order_hash: bytes,
    program_id: Pubkey = PROGRAM_ID,
) -> Tuple[Pubkey, int]:
    """Derive the order status PDA for an order hash.

    Seeds: ["order_status", order_hash]
    """
    return Pubkey.find_program_address(
        [SEED_ORDER_STATUS, order_hash],
        program_id,
    )


def get_user_nonce_pda(
    user: Pubkey,
    program_id: Pubkey = PROGRAM_ID,
) -> Tuple[Pubkey, int]:
    """Derive the user nonce PDA.

    Seeds: ["user_nonce", user]
    """
    return Pubkey.find_program_address(
        [SEED_USER_NONCE, bytes(user)],
        program_id,
    )


def get_position_pda(
    owner: Pubkey,
    market: Pubkey,
    program_id: Pubkey = PROGRAM_ID,
) -> Tuple[Pubkey, int]:
    """Derive the position PDA for a user in a market.

    Seeds: ["position", owner, market]
    """
    return Pubkey.find_program_address(
        [SEED_POSITION, bytes(owner), bytes(market)],
        program_id,
    )


def get_orderbook_pda(
    mint_a: Pubkey,
    mint_b: Pubkey,
    program_id: Pubkey = PROGRAM_ID,
) -> Tuple[Pubkey, int]:
    """Derive the orderbook PDA for a pair of mints.

    Seeds: ["orderbook", mint_a, mint_b]
    """
    return Pubkey.find_program_address(
        [ORDERBOOK_SEED, bytes(mint_a), bytes(mint_b)],
        program_id,
    )


def get_alt_pda(
    orderbook: Pubkey,
    recent_slot: int,
) -> Tuple[Pubkey, int]:
    """Derive the Address Lookup Table PDA.

    Seeds: [orderbook, recent_slot (u64 LE)]
    Uses the ALT_PROGRAM_ID as the program.
    """
    return Pubkey.find_program_address(
        [bytes(orderbook), encode_u64(recent_slot)],
        ALT_PROGRAM_ID,
    )


def get_all_conditional_mints(
    market: Pubkey,
    deposit_mint: Pubkey,
    num_outcomes: int,
    program_id: Pubkey = PROGRAM_ID,
) -> list[Pubkey]:
    """Derive all conditional mint PDAs for a market.

    Returns a list of conditional mint addresses for outcomes 0 to num_outcomes-1.
    """
    return [
        get_conditional_mint_pda(market, deposit_mint, i, program_id)[0]
        for i in range(num_outcomes)
    ]
