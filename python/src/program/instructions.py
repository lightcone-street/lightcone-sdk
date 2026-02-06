"""Instruction builders for the Lightcone SDK.

This module provides functions to build all Lightcone program instructions.
"""

from typing import List

from solders.instruction import AccountMeta, Instruction
from solders.pubkey import Pubkey

from .constants import (
    ALT_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    INSTRUCTION_ACTIVATE_MARKET,
    INSTRUCTION_ADD_DEPOSIT_MINT,
    INSTRUCTION_CANCEL_ORDER,
    INSTRUCTION_CREATE_MARKET,
    INSTRUCTION_CREATE_ORDERBOOK,
    INSTRUCTION_INCREMENT_NONCE,
    INSTRUCTION_INITIALIZE,
    INSTRUCTION_MATCH_ORDERS_MULTI,
    INSTRUCTION_MERGE_COMPLETE_SET,
    INSTRUCTION_MINT_COMPLETE_SET,
    INSTRUCTION_REDEEM_WINNINGS,
    INSTRUCTION_SET_AUTHORITY,
    INSTRUCTION_SET_OPERATOR,
    INSTRUCTION_SET_PAUSED,
    INSTRUCTION_SETTLE_MARKET,
    INSTRUCTION_WITHDRAW_FROM_POSITION,
    MAX_OUTCOME_NAME_LEN,
    MAX_OUTCOME_SYMBOL_LEN,
    MAX_OUTCOME_URI_LEN,
    PROGRAM_ID,
    SYSTEM_PROGRAM_ID,
    TOKEN_2022_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
)
from .orders import hash_order, serialize_order, serialize_full_order, to_order
from .pda import (
    get_conditional_mint_pda,
    get_exchange_pda,
    get_market_pda,
    get_mint_authority_pda,
    get_order_status_pda,
    get_orderbook_pda,
    get_alt_pda,
    get_position_pda,
    get_user_nonce_pda,
    get_vault_pda,
)
from .types import FullOrder, OutcomeMetadata
from .utils import (
    encode_string,
    encode_u64,
    encode_u8,
    get_associated_token_address,
    get_associated_token_address_2022,
)


def build_initialize_instruction(
    authority: Pubkey,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the initialize instruction.

    Accounts:
    0. authority (signer, writable)
    1. exchange (writable)
    2. system_program
    """
    exchange, _ = get_exchange_pda(program_id)

    accounts = [
        AccountMeta(pubkey=authority, is_signer=True, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=True),
        AccountMeta(pubkey=SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
    ]

    data = bytes([INSTRUCTION_INITIALIZE])

    return Instruction(program_id=program_id, accounts=accounts, data=data)


def build_create_market_instruction(
    authority: Pubkey,
    num_outcomes: int,
    oracle: Pubkey,
    question_id: bytes,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the create_market instruction.

    Accounts:
    0. authority (signer, writable)
    1. exchange (writable)
    2. system_program

    Data: [1, num_outcomes (u8), oracle (32), question_id (32)]
    """
    exchange, _ = get_exchange_pda(program_id)

    data = bytearray()
    data.append(INSTRUCTION_CREATE_MARKET)
    data.append(num_outcomes)
    data.extend(bytes(oracle))
    data.extend(question_id)

    accounts = [
        AccountMeta(pubkey=authority, is_signer=True, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=True),
        AccountMeta(pubkey=SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
    ]

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))


def build_create_market_instruction_with_id(
    authority: Pubkey,
    market_id: int,
    num_outcomes: int,
    oracle: Pubkey,
    question_id: bytes,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the create_market instruction with a known market_id.

    Use this when you already know the market_id (from exchange.market_count).
    """
    exchange, _ = get_exchange_pda(program_id)
    market, _ = get_market_pda(market_id, program_id)

    data = bytearray()
    data.append(INSTRUCTION_CREATE_MARKET)
    data.append(num_outcomes)
    data.extend(bytes(oracle))
    data.extend(question_id)

    accounts = [
        AccountMeta(pubkey=authority, is_signer=True, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=True),
        AccountMeta(pubkey=market, is_signer=False, is_writable=True),
        AccountMeta(pubkey=SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
    ]

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))


def build_add_deposit_mint_instruction(
    payer: Pubkey,
    market: Pubkey,
    deposit_mint: Pubkey,
    outcome_metadata: List[OutcomeMetadata],
    num_outcomes: int,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the add_deposit_mint instruction.

    Accounts:
    0. payer (signer, writable)
    1. market (writable)
    2. deposit_mint
    3. vault (writable)
    4. mint_authority
    5. token_program
    6. token_2022_program
    7. system_program
    8+. conditional_mints[0..num_outcomes] (writable)

    Data: [2, ...metadata for each outcome]
    """
    vault, _ = get_vault_pda(deposit_mint, market, program_id)
    mint_authority, _ = get_mint_authority_pda(market, program_id)

    accounts = [
        AccountMeta(pubkey=payer, is_signer=True, is_writable=True),
        AccountMeta(pubkey=market, is_signer=False, is_writable=True),
        AccountMeta(pubkey=deposit_mint, is_signer=False, is_writable=False),
        AccountMeta(pubkey=vault, is_signer=False, is_writable=True),
        AccountMeta(pubkey=mint_authority, is_signer=False, is_writable=False),
        AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(pubkey=TOKEN_2022_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(pubkey=SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
    ]

    # Add conditional mint accounts
    for i in range(num_outcomes):
        cond_mint, _ = get_conditional_mint_pda(market, deposit_mint, i, program_id)
        accounts.append(
            AccountMeta(pubkey=cond_mint, is_signer=False, is_writable=True)
        )

    # Build instruction data
    data = bytearray()
    data.append(INSTRUCTION_ADD_DEPOSIT_MINT)

    # Encode metadata for each outcome
    for meta in outcome_metadata:
        data.extend(encode_string(meta.name, MAX_OUTCOME_NAME_LEN))
        data.extend(encode_string(meta.symbol, MAX_OUTCOME_SYMBOL_LEN))
        data.extend(encode_string(meta.uri, MAX_OUTCOME_URI_LEN))

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))


def build_mint_complete_set_instruction(
    user: Pubkey,
    market: Pubkey,
    deposit_mint: Pubkey,
    amount: int,
    num_outcomes: int,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the mint_complete_set instruction.

    Accounts:
    0. user (signer, writable)
    1. exchange
    2. market
    3. deposit_mint
    4. vault (writable)
    5. user_deposit_ata (writable)
    6. position (writable)
    7. position_collateral_ata (writable)
    8. mint_authority
    9. token_program
    10. token_2022_program
    11. associated_token_program
    12. system_program
    Remaining: [conditional_mint[i], position_conditional_ata[i]] pairs
    """
    exchange, _ = get_exchange_pda(program_id)
    position, _ = get_position_pda(user, market, program_id)
    vault, _ = get_vault_pda(deposit_mint, market, program_id)
    mint_authority, _ = get_mint_authority_pda(market, program_id)

    # Deposit token uses SPL Token, conditional tokens use Token-2022
    user_deposit_ata = get_associated_token_address(user, deposit_mint)
    position_collateral_ata = get_associated_token_address(position, deposit_mint)

    accounts = [
        AccountMeta(pubkey=user, is_signer=True, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=False),
        AccountMeta(pubkey=market, is_signer=False, is_writable=False),
        AccountMeta(pubkey=deposit_mint, is_signer=False, is_writable=False),
        AccountMeta(pubkey=vault, is_signer=False, is_writable=True),
        AccountMeta(pubkey=user_deposit_ata, is_signer=False, is_writable=True),
        AccountMeta(pubkey=position, is_signer=False, is_writable=True),
        AccountMeta(pubkey=position_collateral_ata, is_signer=False, is_writable=True),
        AccountMeta(pubkey=mint_authority, is_signer=False, is_writable=False),
        AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(pubkey=TOKEN_2022_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(pubkey=ASSOCIATED_TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(pubkey=SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
    ]

    # Add conditional mint and position ATA pairs (conditional tokens use Token-2022)
    for i in range(num_outcomes):
        cond_mint, _ = get_conditional_mint_pda(market, deposit_mint, i, program_id)
        position_cond_ata = get_associated_token_address_2022(position, cond_mint)
        accounts.append(
            AccountMeta(pubkey=cond_mint, is_signer=False, is_writable=True)
        )
        accounts.append(
            AccountMeta(pubkey=position_cond_ata, is_signer=False, is_writable=True)
        )

    data = bytearray()
    data.append(INSTRUCTION_MINT_COMPLETE_SET)
    data.extend(encode_u64(amount))

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))


def build_merge_complete_set_instruction(
    user: Pubkey,
    market: Pubkey,
    deposit_mint: Pubkey,
    amount: int,
    num_outcomes: int,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the merge_complete_set instruction.

    Burns all outcome tokens from Position and releases collateral.

    Accounts:
    0. user (signer, writable)
    1. exchange (readonly)
    2. market (readonly)
    3. deposit_mint (readonly)
    4. vault (writable)
    5. position (writable)
    6. user_deposit_ata (writable)
    7. mint_authority (readonly)
    8. token_program (readonly)
    9. token_2022_program (readonly)
    + [conditional_mint, position_ata] pairs
    """
    exchange, _ = get_exchange_pda(program_id)
    position, _ = get_position_pda(user, market, program_id)
    vault, _ = get_vault_pda(deposit_mint, market, program_id)
    mint_authority, _ = get_mint_authority_pda(market, program_id)

    # Deposit token uses SPL Token
    user_deposit_ata = get_associated_token_address(user, deposit_mint)

    accounts = [
        AccountMeta(pubkey=user, is_signer=True, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=False),
        AccountMeta(pubkey=market, is_signer=False, is_writable=False),
        AccountMeta(pubkey=deposit_mint, is_signer=False, is_writable=False),
        AccountMeta(pubkey=vault, is_signer=False, is_writable=True),
        AccountMeta(pubkey=position, is_signer=False, is_writable=True),
        AccountMeta(pubkey=user_deposit_ata, is_signer=False, is_writable=True),
        AccountMeta(pubkey=mint_authority, is_signer=False, is_writable=False),
        AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(pubkey=TOKEN_2022_PROGRAM_ID, is_signer=False, is_writable=False),
    ]

    # Conditional tokens use Token-2022
    for i in range(num_outcomes):
        cond_mint, _ = get_conditional_mint_pda(market, deposit_mint, i, program_id)
        position_cond_ata = get_associated_token_address_2022(position, cond_mint)
        accounts.append(
            AccountMeta(pubkey=cond_mint, is_signer=False, is_writable=True)
        )
        accounts.append(
            AccountMeta(pubkey=position_cond_ata, is_signer=False, is_writable=True)
        )

    data = bytearray()
    data.append(INSTRUCTION_MERGE_COMPLETE_SET)
    data.extend(encode_u64(amount))

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))


def build_cancel_order_instruction(
    maker: Pubkey,
    market: Pubkey,
    order: FullOrder,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the cancel_order instruction.

    Accounts:
    0. maker (signer, writable)
    1. market (readonly)
    2. order_status (writable)
    3. system_program

    Data: [5, order_hash (32), full_order (225)]
    """
    order_hash = hash_order(order)
    order_status, _ = get_order_status_pda(order_hash, program_id)

    accounts = [
        AccountMeta(pubkey=maker, is_signer=True, is_writable=True),
        AccountMeta(pubkey=market, is_signer=False, is_writable=False),
        AccountMeta(pubkey=order_status, is_signer=False, is_writable=True),
        AccountMeta(pubkey=SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
    ]

    data = bytearray()
    data.append(INSTRUCTION_CANCEL_ORDER)
    data.extend(order_hash)
    data.extend(serialize_full_order(order))

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))


def build_increment_nonce_instruction(
    user: Pubkey,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the increment_nonce instruction.

    Accounts:
    0. user (signer, writable)
    1. user_nonce (writable)
    2. system_program
    """
    user_nonce, _ = get_user_nonce_pda(user, program_id)

    accounts = [
        AccountMeta(pubkey=user, is_signer=True, is_writable=True),
        AccountMeta(pubkey=user_nonce, is_signer=False, is_writable=True),
        AccountMeta(pubkey=SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
    ]

    data = bytes([INSTRUCTION_INCREMENT_NONCE])

    return Instruction(program_id=program_id, accounts=accounts, data=data)


def build_settle_market_instruction(
    oracle: Pubkey,
    market_id: int,
    winning_outcome: int,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the settle_market instruction.

    Accounts:
    0. oracle (signer, writable)
    1. exchange
    2. market (writable)

    Data: [7, winning_outcome (u8)]
    """
    exchange, _ = get_exchange_pda(program_id)
    market, _ = get_market_pda(market_id, program_id)

    accounts = [
        AccountMeta(pubkey=oracle, is_signer=True, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=False),
        AccountMeta(pubkey=market, is_signer=False, is_writable=True),
    ]

    data = bytes([INSTRUCTION_SETTLE_MARKET, winning_outcome])

    return Instruction(program_id=program_id, accounts=accounts, data=data)


def build_redeem_winnings_instruction(
    user: Pubkey,
    market: Pubkey,
    deposit_mint: Pubkey,
    winning_outcome: int,
    amount: int,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the redeem_winnings instruction.

    Accounts:
    0. user (signer, writable)
    1. market
    2. deposit_mint
    3. vault (writable)
    4. winning_mint (writable)
    5. position (writable)
    6. position_winning_ata (writable)
    7. user_deposit_ata (writable)
    8. mint_authority
    9. token_program
    10. token_2022_program
    """
    position, _ = get_position_pda(user, market, program_id)
    vault, _ = get_vault_pda(deposit_mint, market, program_id)
    mint_authority, _ = get_mint_authority_pda(market, program_id)
    winning_mint, _ = get_conditional_mint_pda(
        market, deposit_mint, winning_outcome, program_id
    )

    position_winning_ata = get_associated_token_address_2022(position, winning_mint)
    user_deposit_ata = get_associated_token_address(user, deposit_mint)

    accounts = [
        AccountMeta(pubkey=user, is_signer=True, is_writable=True),
        AccountMeta(pubkey=market, is_signer=False, is_writable=False),
        AccountMeta(pubkey=deposit_mint, is_signer=False, is_writable=False),
        AccountMeta(pubkey=vault, is_signer=False, is_writable=True),
        AccountMeta(pubkey=winning_mint, is_signer=False, is_writable=True),
        AccountMeta(pubkey=position, is_signer=False, is_writable=True),
        AccountMeta(pubkey=position_winning_ata, is_signer=False, is_writable=True),
        AccountMeta(pubkey=user_deposit_ata, is_signer=False, is_writable=True),
        AccountMeta(pubkey=mint_authority, is_signer=False, is_writable=False),
        AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(pubkey=TOKEN_2022_PROGRAM_ID, is_signer=False, is_writable=False),
    ]

    data = bytearray()
    data.append(INSTRUCTION_REDEEM_WINNINGS)
    data.extend(encode_u64(amount))

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))


def build_set_paused_instruction(
    authority: Pubkey,
    paused: bool,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the set_paused instruction.

    Accounts:
    0. authority (signer, writable)
    1. exchange (writable)

    Data: [9, paused (0 or 1)]
    """
    exchange, _ = get_exchange_pda(program_id)

    accounts = [
        AccountMeta(pubkey=authority, is_signer=True, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=True),
    ]

    data = bytes([INSTRUCTION_SET_PAUSED, 1 if paused else 0])

    return Instruction(program_id=program_id, accounts=accounts, data=data)


def build_set_operator_instruction(
    authority: Pubkey,
    new_operator: Pubkey,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the set_operator instruction.

    Accounts:
    0. authority (signer, writable)
    1. exchange (writable)

    Data: [10, new_operator (32)]
    """
    exchange, _ = get_exchange_pda(program_id)

    accounts = [
        AccountMeta(pubkey=authority, is_signer=True, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=True),
    ]

    data = bytearray()
    data.append(INSTRUCTION_SET_OPERATOR)
    data.extend(bytes(new_operator))

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))


def build_withdraw_from_position_instruction(
    user: Pubkey,
    market: Pubkey,
    mint: Pubkey,
    amount: int,
    outcome_index: int,
    is_token_2022: bool = True,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the withdraw_from_position instruction.

    Accounts:
    0. user (signer, writable)
    1. market (readonly)
    2. position (writable)
    3. mint (readonly)
    4. position_ata (writable)
    5. user_ata (writable)
    6. token_program (SPL Token or Token-2022)
    """
    position, _ = get_position_pda(user, market, program_id)
    token_program = TOKEN_2022_PROGRAM_ID if is_token_2022 else TOKEN_PROGRAM_ID

    if is_token_2022:
        position_ata = get_associated_token_address_2022(position, mint)
        user_ata = get_associated_token_address_2022(user, mint)
    else:
        position_ata = get_associated_token_address(position, mint)
        user_ata = get_associated_token_address(user, mint)

    accounts = [
        AccountMeta(pubkey=user, is_signer=True, is_writable=True),
        AccountMeta(pubkey=market, is_signer=False, is_writable=False),
        AccountMeta(pubkey=position, is_signer=False, is_writable=True),
        AccountMeta(pubkey=mint, is_signer=False, is_writable=False),
        AccountMeta(pubkey=position_ata, is_signer=False, is_writable=True),
        AccountMeta(pubkey=user_ata, is_signer=False, is_writable=True),
        AccountMeta(pubkey=token_program, is_signer=False, is_writable=False),
    ]

    data = bytearray()
    data.append(INSTRUCTION_WITHDRAW_FROM_POSITION)
    data.extend(encode_u64(amount))
    data.extend(encode_u8(outcome_index))

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))


def build_activate_market_instruction(
    authority: Pubkey,
    market_id: int,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the activate_market instruction.

    Accounts:
    0. authority (signer, writable)
    1. exchange
    2. market (writable)
    """
    exchange, _ = get_exchange_pda(program_id)
    market, _ = get_market_pda(market_id, program_id)

    accounts = [
        AccountMeta(pubkey=authority, is_signer=True, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=False),
        AccountMeta(pubkey=market, is_signer=False, is_writable=True),
    ]

    data = bytes([INSTRUCTION_ACTIVATE_MARKET])

    return Instruction(program_id=program_id, accounts=accounts, data=data)


def build_match_orders_multi_instruction(
    operator: Pubkey,
    market: Pubkey,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    taker_order: FullOrder,
    maker_orders: List[FullOrder],
    maker_fill_amounts: List[int],
    taker_fill_amounts: List[int],
    full_fill_bitmask: int = 0,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the match_orders_multi instruction.

    New data format:
    - discriminator(1) + taker_order(29) + taker_sig(64) + num_makers(1) + bitmask(1)
    - Per maker: order(29) + sig(64) + maker_fill(8) + taker_fill(8) = 109 bytes each

    Accounts:
    0. operator (signer, writable)
    1. exchange
    2. market
    3. taker_order_status (writable)
    4. taker_nonce
    5. taker_position (writable)
    6. base_mint
    7. quote_mint
    8. taker_position_base_ata (writable)
    9. taker_position_quote_ata (writable)
    10. token_2022_program
    11. system_program
    Per maker (5 accounts each, conditionally including order_status based on bitmask):
    - order_status (writable) [only if bit set in bitmask]
    - nonce
    - position (writable)
    - base_ata (writable)
    - quote_ata (writable)
    """
    exchange, _ = get_exchange_pda(program_id)

    taker_hash = hash_order(taker_order)
    taker_nonce, _ = get_user_nonce_pda(taker_order.maker, program_id)
    taker_position, _ = get_position_pda(taker_order.maker, market, program_id)

    taker_position_base_ata = get_associated_token_address_2022(taker_position, base_mint)
    taker_position_quote_ata = get_associated_token_address_2022(taker_position, quote_mint)

    taker_full_fill = bool((full_fill_bitmask >> 7) & 1)

    accounts = [
        AccountMeta(pubkey=operator, is_signer=True, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=False),
        AccountMeta(pubkey=market, is_signer=False, is_writable=False),
    ]

    # Taker order_status: only if NOT full fill (bit 7 = 0)
    if not taker_full_fill:
        taker_order_status, _ = get_order_status_pda(taker_hash, program_id)
        accounts.append(
            AccountMeta(pubkey=taker_order_status, is_signer=False, is_writable=True)
        )

    accounts.extend([
        AccountMeta(pubkey=taker_nonce, is_signer=False, is_writable=False),
        AccountMeta(pubkey=taker_position, is_signer=False, is_writable=True),
        AccountMeta(pubkey=base_mint, is_signer=False, is_writable=False),
        AccountMeta(pubkey=quote_mint, is_signer=False, is_writable=False),
        AccountMeta(pubkey=taker_position_base_ata, is_signer=False, is_writable=True),
        AccountMeta(pubkey=taker_position_quote_ata, is_signer=False, is_writable=True),
        AccountMeta(pubkey=TOKEN_2022_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(pubkey=SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
    ])

    # Add maker accounts
    num_makers = len(maker_orders)
    for i, maker_order in enumerate(maker_orders):
        # bit i = 0 means NOT full fill -> INCLUDE order_status
        maker_full_fill = bool((full_fill_bitmask >> i) & 1)

        if not maker_full_fill:
            maker_hash = hash_order(maker_order)
            maker_order_status, _ = get_order_status_pda(maker_hash, program_id)
            accounts.append(
                AccountMeta(pubkey=maker_order_status, is_signer=False, is_writable=True)
            )

        maker_nonce, _ = get_user_nonce_pda(maker_order.maker, program_id)
        maker_position, _ = get_position_pda(maker_order.maker, market, program_id)
        maker_position_base_ata = get_associated_token_address_2022(maker_position, base_mint)
        maker_position_quote_ata = get_associated_token_address_2022(maker_position, quote_mint)

        accounts.extend([
            AccountMeta(pubkey=maker_nonce, is_signer=False, is_writable=False),
            AccountMeta(pubkey=maker_position, is_signer=False, is_writable=True),
            AccountMeta(pubkey=maker_position_base_ata, is_signer=False, is_writable=True),
            AccountMeta(pubkey=maker_position_quote_ata, is_signer=False, is_writable=True),
        ])

    # Build instruction data
    data = bytearray()
    data.append(INSTRUCTION_MATCH_ORDERS_MULTI)

    # Taker data: order(29) + sig(64)
    taker_compact = to_order(taker_order)
    data.extend(serialize_order(taker_compact))
    data.extend(taker_order.signature)

    # Number of makers + bitmask
    data.append(num_makers)
    data.append(full_fill_bitmask & 0xFF)

    # Maker data: order(29) + sig(64) + maker_fill(8) + taker_fill(8) per maker
    for i, maker_order in enumerate(maker_orders):
        maker_compact = to_order(maker_order)
        data.extend(serialize_order(maker_compact))
        data.extend(maker_order.signature)
        data.extend(encode_u64(maker_fill_amounts[i]))
        data.extend(encode_u64(taker_fill_amounts[i]))

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))


def build_create_orderbook_instruction(
    payer: Pubkey,
    market: Pubkey,
    mint_a: Pubkey,
    mint_b: Pubkey,
    recent_slot: int,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the create_orderbook instruction.

    Accounts:
    0. payer (signer, writable)
    1. market (readonly)
    2. mint_a (readonly)
    3. mint_b (readonly)
    4. orderbook (writable)
    5. lookup_table (writable)
    6. exchange (readonly)
    7. alt_program (readonly)
    8. system_program

    Data: [15, recent_slot (u64)]
    """
    exchange, _ = get_exchange_pda(program_id)
    orderbook, _ = get_orderbook_pda(mint_a, mint_b, program_id)
    lookup_table, _ = get_alt_pda(orderbook, recent_slot)

    accounts = [
        AccountMeta(pubkey=payer, is_signer=True, is_writable=True),
        AccountMeta(pubkey=market, is_signer=False, is_writable=False),
        AccountMeta(pubkey=mint_a, is_signer=False, is_writable=False),
        AccountMeta(pubkey=mint_b, is_signer=False, is_writable=False),
        AccountMeta(pubkey=orderbook, is_signer=False, is_writable=True),
        AccountMeta(pubkey=lookup_table, is_signer=False, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=False),
        AccountMeta(pubkey=ALT_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(pubkey=SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
    ]

    data = bytearray()
    data.append(INSTRUCTION_CREATE_ORDERBOOK)
    data.extend(encode_u64(recent_slot))

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))


def build_set_authority_instruction(
    current_authority: Pubkey,
    new_authority: Pubkey,
    program_id: Pubkey = PROGRAM_ID,
) -> Instruction:
    """Build the set_authority instruction.

    Accounts:
    0. authority (signer, writable)
    1. exchange (writable)

    Data: [14, new_authority (32)]
    """
    exchange, _ = get_exchange_pda(program_id)

    accounts = [
        AccountMeta(pubkey=current_authority, is_signer=True, is_writable=True),
        AccountMeta(pubkey=exchange, is_signer=False, is_writable=True),
    ]

    data = bytearray()
    data.append(INSTRUCTION_SET_AUTHORITY)
    data.extend(bytes(new_authority))

    return Instruction(program_id=program_id, accounts=accounts, data=bytes(data))
