//! Instruction builders for all Lightcone Pinocchio instructions.
//!
//! This module provides functions to build transaction instructions for interacting
//! with the Lightcone Pinocchio program.

use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

// System program ID
fn system_program_id() -> Pubkey {
    solana_system_interface::program::ID
}

use crate::program::constants::{
    instruction, ALT_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, MAX_MAKERS, TOKEN_2022_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
};
use crate::program::error::{SdkError, SdkResult};
use crate::program::orders::SignedOrder;
use crate::program::pda::{
    get_conditional_mint_pda, get_exchange_pda, get_global_deposit_token_pda, get_market_pda,
    get_mint_authority_pda, get_order_status_pda, get_orderbook_pda, get_alt_pda,
    get_position_alt_pda, get_position_pda, get_user_global_deposit_pda, get_user_nonce_pda,
    get_vault_pda,
};
use crate::program::types::{
    ActivateMarketParams, AddDepositMintParams, CreateMarketParams, CreateOrderbookParams,
    DepositAndSwapParams, DepositToGlobalParams, GlobalToMarketDepositParams,
    InitPositionTokensParams, MatchOrdersMultiParams, MergeCompleteSetParams,
    MintCompleteSetParams, RedeemWinningsParams, SetAuthorityParams, SettleMarketParams,
    WhitelistDepositTokenParams, WithdrawFromPositionParams,
};
use crate::program::utils::{
    get_conditional_token_ata, get_deposit_token_ata, serialize_outcome_metadata,
    validate_outcome_count, OutcomeMetadataInput,
};

// ============================================================================
// Helper Functions
// ============================================================================

/// Create an account meta for a signer+writable account.
fn signer_mut(pubkey: Pubkey) -> AccountMeta {
    AccountMeta::new(pubkey, true)
}

/// Create an account meta for a writable account.
fn writable(pubkey: Pubkey) -> AccountMeta {
    AccountMeta::new(pubkey, false)
}

/// Create an account meta for a read-only account.
fn readonly(pubkey: Pubkey) -> AccountMeta {
    AccountMeta::new_readonly(pubkey, false)
}

// ============================================================================
// Instruction Builders
// ============================================================================

/// Build Initialize instruction.
///
/// Creates the exchange account (singleton).
///
/// Accounts:
/// 0. authority (signer, mut) - Initial admin
/// 1. exchange (mut) - Exchange PDA
/// 2. system_program (readonly)
pub fn build_initialize_ix(authority: &Pubkey, program_id: &Pubkey) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![
        signer_mut(*authority),
        writable(exchange),
        readonly(system_program_id()),
    ];

    let data = vec![instruction::INITIALIZE];

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build CreateMarket instruction.
///
/// Creates a new market in Pending status.
///
/// Accounts:
/// 0. authority (signer, mut) - Must be exchange authority
/// 1. exchange (mut) - Exchange PDA
/// 2. market (mut) - Market PDA
/// 3. system_program (readonly)
pub fn build_create_market_ix(
    params: &CreateMarketParams,
    market_id: u64,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    validate_outcome_count(params.num_outcomes)?;

    let (exchange, _) = get_exchange_pda(program_id);
    let (market, _) = get_market_pda(market_id, program_id);

    let keys = vec![
        signer_mut(params.authority),
        writable(exchange),
        writable(market),
        readonly(system_program_id()),
    ];

    // Data: [discriminator, num_outcomes (u8), oracle (32), question_id (32)]
    let mut data = Vec::with_capacity(66);
    data.push(instruction::CREATE_MARKET);
    data.push(params.num_outcomes);
    data.extend_from_slice(params.oracle.as_ref());
    data.extend_from_slice(&params.question_id);

    Ok(Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    })
}

/// Build AddDepositMint instruction.
///
/// Sets up vault and conditional mints for a deposit token.
///
/// Accounts:
/// 0. payer (signer)
/// 1. market
/// 2. deposit_mint
/// 3. vault
/// 4. mint_authority
/// 5. token_program (SPL Token)
/// 6. token_2022_program
/// 7. system_program
/// 8. conditional_mints\[0..num_outcomes\]
pub fn build_add_deposit_mint_ix(
    params: &AddDepositMintParams,
    market: &Pubkey,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.outcome_metadata.len() != num_outcomes as usize {
        return Err(SdkError::InvalidOutcomeCount { count: params.outcome_metadata.len() as u8 });
    }

    let (vault, _) = get_vault_pda(&params.deposit_mint, market, program_id);
    let (mint_authority, _) = get_mint_authority_pda(market, program_id);

    let mut keys = vec![
        signer_mut(params.payer),
        writable(*market),
        readonly(params.deposit_mint),
        writable(vault),
        readonly(mint_authority),
        readonly(TOKEN_PROGRAM_ID),
        readonly(TOKEN_2022_PROGRAM_ID),
        readonly(system_program_id()),
    ];

    // Add conditional mints
    for i in 0..num_outcomes {
        let (mint, _) = get_conditional_mint_pda(market, &params.deposit_mint, i, program_id);
        keys.push(writable(mint));
    }

    // Convert OutcomeMetadata to OutcomeMetadataInput
    let metadata_input: Vec<OutcomeMetadataInput> = params
        .outcome_metadata
        .iter()
        .map(|m| OutcomeMetadataInput {
            name: m.name.clone(),
            symbol: m.symbol.clone(),
            uri: m.uri.clone(),
        })
        .collect();

    let mut data = vec![instruction::ADD_DEPOSIT_MINT];
    data.extend(serialize_outcome_metadata(&metadata_input));

    Ok(Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    })
}

/// Build MintCompleteSet instruction.
///
/// Deposits collateral and mints all outcome tokens into Position PDA.
///
/// Accounts:
/// 0. user (signer)
/// 1. exchange
/// 2. market
/// 3. deposit_mint
/// 4. vault
/// 5. user_deposit_ata
/// 6. position
/// 7. position_collateral_ata
/// 8. mint_authority
/// 9. token_program
/// 10. token_2022_program
/// 11. associated_token_program
/// 12. system_program
/// + remaining accounts (conditional_mint, position_conditional_ata) pairs
pub fn build_mint_complete_set_ix(
    params: &MintCompleteSetParams,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (vault, _) = get_vault_pda(&params.deposit_mint, &params.market, program_id);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);
    let user_deposit_ata = get_deposit_token_ata(&params.user, &params.deposit_mint);
    let position_collateral_ata = get_deposit_token_ata(&position, &params.deposit_mint);

    let mut keys = vec![
        signer_mut(params.user),
        readonly(exchange),
        readonly(params.market),
        readonly(params.deposit_mint),
        writable(vault),
        writable(user_deposit_ata),
        writable(position),
        writable(position_collateral_ata),
        readonly(mint_authority),
        readonly(TOKEN_PROGRAM_ID),
        readonly(TOKEN_2022_PROGRAM_ID),
        readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
        readonly(system_program_id()),
    ];

    // Add conditional mint and position ATA pairs
    for i in 0..num_outcomes {
        let (mint, _) =
            get_conditional_mint_pda(&params.market, &params.deposit_mint, i, program_id);
        keys.push(writable(mint));
        let position_ata = get_conditional_token_ata(&position, &mint);
        keys.push(writable(position_ata));
    }

    // Data: [discriminator, amount (u64)]
    let mut data = Vec::with_capacity(9);
    data.push(instruction::MINT_COMPLETE_SET);
    data.extend_from_slice(&params.amount.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build MergeCompleteSet instruction.
///
/// Burns all outcome tokens from Position and releases collateral.
pub fn build_merge_complete_set_ix(
    params: &MergeCompleteSetParams,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (vault, _) = get_vault_pda(&params.deposit_mint, &params.market, program_id);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);
    let user_deposit_ata = get_deposit_token_ata(&params.user, &params.deposit_mint);

    let mut keys = vec![
        signer_mut(params.user),
        readonly(exchange),
        readonly(params.market),
        readonly(params.deposit_mint),
        writable(vault),
        writable(position),
        writable(user_deposit_ata),
        readonly(mint_authority),
        readonly(TOKEN_PROGRAM_ID),
        readonly(TOKEN_2022_PROGRAM_ID),
    ];

    // Add conditional mint and position ATA pairs
    for i in 0..num_outcomes {
        let (mint, _) =
            get_conditional_mint_pda(&params.market, &params.deposit_mint, i, program_id);
        keys.push(writable(mint));
        let position_ata = get_conditional_token_ata(&position, &mint);
        keys.push(writable(position_ata));
    }

    let mut data = Vec::with_capacity(9);
    data.push(instruction::MERGE_COMPLETE_SET);
    data.extend_from_slice(&params.amount.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build CancelOrder instruction.
///
/// Marks an order as cancelled.
///
/// Accounts:
/// 0. maker (signer)
/// 1. market (readonly)
/// 2. order_status (mut)
/// 3. system_program (readonly)
pub fn build_cancel_order_ix(
    maker: &Pubkey,
    market: &Pubkey,
    order: &SignedOrder,
    program_id: &Pubkey,
) -> Instruction {
    let order_hash = order.hash();
    let (order_status, _) = get_order_status_pda(&order_hash, program_id);

    let keys = vec![
        signer_mut(*maker),
        readonly(*market),
        writable(order_status),
        readonly(system_program_id()),
    ];

    // Data: [discriminator(1), order_hash(32), SignedOrder(225)] = 258 bytes
    let mut data = Vec::with_capacity(258);
    data.push(instruction::CANCEL_ORDER);
    data.extend_from_slice(&order_hash);
    data.extend_from_slice(&order.serialize());

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build IncrementNonce instruction.
///
/// Increments user's nonce for replay protection / mass cancellation.
pub fn build_increment_nonce_ix(user: &Pubkey, program_id: &Pubkey) -> Instruction {
    let (user_nonce, _) = get_user_nonce_pda(user, program_id);

    let keys = vec![
        signer_mut(*user),
        writable(user_nonce),
        readonly(system_program_id()),
    ];

    let data = vec![instruction::INCREMENT_NONCE];

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build SettleMarket instruction.
///
/// Oracle resolves the market with winning outcome.
pub fn build_settle_market_ix(params: &SettleMarketParams, program_id: &Pubkey) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (market, _) = get_market_pda(params.market_id, program_id);

    let keys = vec![
        signer_mut(params.oracle),
        readonly(exchange),
        writable(market),
    ];

    let data = vec![instruction::SETTLE_MARKET, params.winning_outcome];

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build RedeemWinnings instruction.
///
/// Redeem winning outcome tokens from Position for collateral.
pub fn build_redeem_winnings_ix(
    params: &RedeemWinningsParams,
    winning_outcome: u8,
    program_id: &Pubkey,
) -> Instruction {
    let (vault, _) = get_vault_pda(&params.deposit_mint, &params.market, program_id);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);
    let (winning_mint, _) = get_conditional_mint_pda(
        &params.market,
        &params.deposit_mint,
        winning_outcome,
        program_id,
    );
    let position_winning_ata = get_conditional_token_ata(&position, &winning_mint);
    let user_deposit_ata = get_deposit_token_ata(&params.user, &params.deposit_mint);

    let keys = vec![
        signer_mut(params.user),
        readonly(params.market),
        readonly(params.deposit_mint),
        writable(vault),
        writable(winning_mint),
        writable(position),
        writable(position_winning_ata),
        writable(user_deposit_ata),
        readonly(mint_authority),
        readonly(TOKEN_PROGRAM_ID),
        readonly(TOKEN_2022_PROGRAM_ID),
    ];

    let mut data = Vec::with_capacity(9);
    data.push(instruction::REDEEM_WINNINGS);
    data.extend_from_slice(&params.amount.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build SetPaused instruction.
///
/// Admin: pause/unpause exchange.
pub fn build_set_paused_ix(
    authority: &Pubkey,
    paused: bool,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![signer_mut(*authority), writable(exchange)];

    let data = vec![instruction::SET_PAUSED, if paused { 1 } else { 0 }];

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build SetOperator instruction.
///
/// Admin: change operator pubkey.
pub fn build_set_operator_ix(
    authority: &Pubkey,
    new_operator: &Pubkey,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![signer_mut(*authority), writable(exchange)];

    let mut data = Vec::with_capacity(33);
    data.push(instruction::SET_OPERATOR);
    data.extend_from_slice(new_operator.as_ref());

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build WithdrawFromPosition instruction.
///
/// Withdraw tokens from Position ATA to user's ATA.
///
/// Accounts (7):
/// 0. user (signer)
/// 1. market (readonly)
/// 2. position (mut)
/// 3. mint (readonly)
/// 4. position_ata (mut)
/// 5. user_ata (mut)
/// 6. token_program (readonly)
pub fn build_withdraw_from_position_ix(
    params: &WithdrawFromPositionParams,
    is_token_2022: bool,
    program_id: &Pubkey,
) -> Instruction {
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);
    let position_ata = if is_token_2022 {
        get_conditional_token_ata(&position, &params.mint)
    } else {
        get_deposit_token_ata(&position, &params.mint)
    };
    let user_ata = if is_token_2022 {
        get_conditional_token_ata(&params.user, &params.mint)
    } else {
        get_deposit_token_ata(&params.user, &params.mint)
    };
    let token_program = if is_token_2022 {
        TOKEN_2022_PROGRAM_ID
    } else {
        TOKEN_PROGRAM_ID
    };

    let keys = vec![
        signer_mut(params.user),
        readonly(params.market),
        writable(position),
        readonly(params.mint),
        writable(position_ata),
        writable(user_ata),
        readonly(token_program),
    ];

    // Data: [discriminator(1), amount(8), outcome_index(1)] = 10 bytes
    let mut data = Vec::with_capacity(10);
    data.push(instruction::WITHDRAW_FROM_POSITION);
    data.extend_from_slice(&params.amount.to_le_bytes());
    data.push(params.outcome_index);

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build ActivateMarket instruction.
///
/// Authority: Pending → Active.
pub fn build_activate_market_ix(
    params: &ActivateMarketParams,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (market, _) = get_market_pda(params.market_id, program_id);

    let keys = vec![
        signer_mut(params.authority),
        readonly(exchange),
        writable(market),
    ];

    let data = vec![instruction::ACTIVATE_MARKET];

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build MatchOrdersMulti instruction.
///
/// Match taker against up to 3 makers.
///
/// Data format:
/// [0]       discriminator
/// [1..30]   taker Order (29 bytes)
/// [30..94]  taker_signature (64 bytes)
/// [94]      num_makers
/// [95]      full_fill_bitmask
/// Per maker (109 bytes each):
///   [+0..+29]   maker Order (29)
///   [+29..+93]  maker_signature (64)
///   [+93..+101] maker_fill_amount (8)
///   [+101..+109] taker_fill_amount (8)
///
/// Account construction uses bitmask to determine if order_status is included.
pub fn build_match_orders_multi_ix(
    params: &MatchOrdersMultiParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.maker_orders.is_empty() {
        return Err(SdkError::MissingField("maker_orders".to_string()));
    }
    if params.maker_orders.len() > MAX_MAKERS {
        return Err(SdkError::TooManyMakers { count: params.maker_orders.len() });
    }
    if params.maker_orders.len() != params.maker_fill_amounts.len() {
        return Err(SdkError::MissingField("maker_fill_amounts".to_string()));
    }
    if params.maker_orders.len() != params.taker_fill_amounts.len() {
        return Err(SdkError::MissingField("taker_fill_amounts".to_string()));
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let taker_order_hash = params.taker_order.hash();
    let (taker_nonce, _) = get_user_nonce_pda(&params.taker_order.maker, program_id);
    let (taker_position, _) =
        get_position_pda(&params.taker_order.maker, &params.market, program_id);
    let taker_base_ata = get_conditional_token_ata(&taker_position, &params.base_mint);
    let taker_quote_ata = get_conditional_token_ata(&taker_position, &params.quote_mint);

    let taker_full_fill = (params.full_fill_bitmask >> 7) & 1 == 1;

    let mut keys = Vec::new();

    // Taker fixed accounts
    keys.push(signer_mut(params.operator));
    keys.push(readonly(exchange));
    keys.push(readonly(params.market));

    if !taker_full_fill {
        // bit 7 = 0: needs order_status (12 accounts)
        let (taker_order_status, _) = get_order_status_pda(&taker_order_hash, program_id);
        keys.push(writable(taker_order_status));
    }
    // Remaining taker accounts
    keys.push(readonly(taker_nonce));
    keys.push(writable(taker_position));
    keys.push(readonly(params.base_mint));
    keys.push(readonly(params.quote_mint));
    keys.push(writable(taker_base_ata));
    keys.push(writable(taker_quote_ata));
    keys.push(readonly(TOKEN_2022_PROGRAM_ID));
    keys.push(readonly(system_program_id()));

    // Per-maker accounts
    for (i, maker_order) in params.maker_orders.iter().enumerate() {
        let maker_full_fill = (params.full_fill_bitmask >> i) & 1 == 1;

        if !maker_full_fill {
            // bit i = 0: 5 accounts (order_status, nonce, position, base_ata, quote_ata)
            let maker_order_hash = maker_order.hash();
            let (maker_order_status, _) = get_order_status_pda(&maker_order_hash, program_id);
            keys.push(writable(maker_order_status));
        }
        // bit i = 1: 4 accounts (nonce, position, base_ata, quote_ata)
        let (maker_nonce, _) = get_user_nonce_pda(&maker_order.maker, program_id);
        let (maker_position, _) = get_position_pda(&maker_order.maker, &params.market, program_id);
        let maker_base_ata = get_conditional_token_ata(&maker_position, &params.base_mint);
        let maker_quote_ata = get_conditional_token_ata(&maker_position, &params.quote_mint);

        keys.push(readonly(maker_nonce));
        keys.push(writable(maker_position));
        keys.push(writable(maker_base_ata));
        keys.push(writable(maker_quote_ata));
    }

    // Build data
    let taker_compact = params.taker_order.to_order();
    let num_makers = params.maker_orders.len() as u8;

    // discriminator(1) + taker_order(29) + taker_sig(64) + num_makers(1) + bitmask(1)
    // + per maker: order(29) + sig(64) + maker_fill(8) + taker_fill(8) = 109
    let data_size = 1 + 29 + 64 + 1 + 1 + (params.maker_orders.len() * 109);
    let mut data = Vec::with_capacity(data_size);

    data.push(instruction::MATCH_ORDERS_MULTI);
    data.extend_from_slice(&taker_compact.serialize());
    data.extend_from_slice(&params.taker_order.signature);
    data.push(num_makers);
    data.push(params.full_fill_bitmask);

    for (i, maker_order) in params.maker_orders.iter().enumerate() {
        let maker_compact = maker_order.to_order();

        data.extend_from_slice(&maker_compact.serialize());
        data.extend_from_slice(&maker_order.signature);
        data.extend_from_slice(&params.maker_fill_amounts[i].to_le_bytes());
        data.extend_from_slice(&params.taker_fill_amounts[i].to_le_bytes());
    }

    Ok(Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    })
}

/// Build CreateOrderbook instruction.
///
/// Creates an on-chain orderbook with address lookup table.
///
/// Accounts (9):
/// 0. payer (signer, mut)
/// 1. market (readonly)
/// 2. mint_a (readonly)
/// 3. mint_b (readonly)
/// 4. orderbook (mut)
/// 5. lookup_table (mut)
/// 6. exchange (readonly)
/// 7. alt_program (readonly)
/// 8. system_program (readonly)
pub fn build_create_orderbook_ix(
    params: &CreateOrderbookParams,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (orderbook, _) = get_orderbook_pda(&params.mint_a, &params.mint_b, program_id);
    let (lookup_table, _) = get_alt_pda(&orderbook, params.recent_slot);

    let keys = vec![
        signer_mut(params.payer),
        readonly(params.market),
        readonly(params.mint_a),
        readonly(params.mint_b),
        writable(orderbook),
        writable(lookup_table),
        readonly(exchange),
        readonly(*ALT_PROGRAM_ID),
        readonly(system_program_id()),
    ];

    // Data: [discriminator(1), recent_slot(8)] = 9 bytes
    let mut data = Vec::with_capacity(9);
    data.push(instruction::CREATE_ORDERBOOK);
    data.extend_from_slice(&params.recent_slot.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build SetAuthority instruction.
///
/// Change the exchange authority.
///
/// Accounts (2):
/// 0. authority (signer)
/// 1. exchange (mut)
pub fn build_set_authority_ix(
    params: &SetAuthorityParams,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![signer_mut(params.current_authority), writable(exchange)];

    // Data: [discriminator(1), new_authority(32)] = 33 bytes
    let mut data = Vec::with_capacity(33);
    data.push(instruction::SET_AUTHORITY);
    data.extend_from_slice(params.new_authority.as_ref());

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build WhitelistDepositToken instruction.
///
/// Admin: whitelist a token mint for global deposits.
///
/// Accounts (5):
/// 0. authority (signer, mut) - Must be exchange authority
/// 1. exchange (readonly) - Exchange PDA
/// 2. mint (readonly) - Token mint to whitelist
/// 3. global_deposit_token (mut) - PDA to create ["global_deposit", mint]
/// 4. system_program (readonly)
pub fn build_whitelist_deposit_token_ix(
    params: &WhitelistDepositTokenParams,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (global_deposit_token, _) = get_global_deposit_token_pda(&params.mint, program_id);

    let keys = vec![
        signer_mut(params.authority),
        readonly(exchange),
        readonly(params.mint),
        writable(global_deposit_token),
        readonly(system_program_id()),
    ];

    let data = vec![instruction::WHITELIST_DEPOSIT_TOKEN];

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build DepositToGlobal instruction.
///
/// Deposit tokens from user's token account into their global deposit PDA.
///
/// Accounts (7):
/// 0. user (signer, mut)
/// 1. global_deposit_token (readonly) - Whitelist PDA
/// 2. mint (readonly)
/// 3. user_global_deposit (mut) - User's deposit PDA
/// 4. user_token_account (mut) - User's source token account
/// 5. token_program (readonly)
/// 6. system_program (readonly)
pub fn build_deposit_to_global_ix(
    params: &DepositToGlobalParams,
    program_id: &Pubkey,
) -> Instruction {
    let (global_deposit_token, _) = get_global_deposit_token_pda(&params.mint, program_id);
    let (user_global_deposit, _) =
        get_user_global_deposit_pda(&params.user, &params.mint, program_id);
    let user_token_account = get_deposit_token_ata(&params.user, &params.mint);

    let keys = vec![
        signer_mut(params.user),
        readonly(global_deposit_token),
        readonly(params.mint),
        writable(user_global_deposit),
        writable(user_token_account),
        readonly(TOKEN_PROGRAM_ID),
        readonly(system_program_id()),
    ];

    let mut data = Vec::with_capacity(9);
    data.push(instruction::DEPOSIT_TO_GLOBAL);
    data.extend_from_slice(&params.amount.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build GlobalToMarketDeposit instruction.
///
/// Transfer from user's global deposit to market vault + mint conditional tokens.
///
/// Accounts (14 + num_outcomes*2):
/// 0. user (signer, mut)
/// 1. exchange (readonly)
/// 2. market (readonly)
/// 3. deposit_mint (readonly)
/// 4. vault (mut)
/// 5. global_deposit_token (readonly)
/// 6. user_global_deposit (mut)
/// 7. position (mut)
/// 8. position_collateral_ata (mut)
/// 9. mint_authority (readonly)
/// 10. token_program (readonly)
/// 11. token_2022_program (readonly)
/// 12. ata_program (readonly)
/// 13. system_program (readonly)
/// + per outcome: conditional_mint[i] (mut), position_conditional_ata[i] (mut)
pub fn build_global_to_market_deposit_ix(
    params: &GlobalToMarketDepositParams,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (vault, _) = get_vault_pda(&params.deposit_mint, &params.market, program_id);
    let (global_deposit_token, _) =
        get_global_deposit_token_pda(&params.deposit_mint, program_id);
    let (user_global_deposit, _) =
        get_user_global_deposit_pda(&params.user, &params.deposit_mint, program_id);
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);
    let position_collateral_ata = get_deposit_token_ata(&position, &params.deposit_mint);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);

    let mut keys = vec![
        signer_mut(params.user),
        readonly(exchange),
        readonly(params.market),
        readonly(params.deposit_mint),
        writable(vault),
        readonly(global_deposit_token),
        writable(user_global_deposit),
        writable(position),
        writable(position_collateral_ata),
        readonly(mint_authority),
        readonly(TOKEN_PROGRAM_ID),
        readonly(TOKEN_2022_PROGRAM_ID),
        readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
        readonly(system_program_id()),
    ];

    for i in 0..num_outcomes {
        let (mint, _) =
            get_conditional_mint_pda(&params.market, &params.deposit_mint, i, program_id);
        keys.push(writable(mint));
        let position_ata = get_conditional_token_ata(&position, &mint);
        keys.push(writable(position_ata));
    }

    let mut data = Vec::with_capacity(9);
    data.push(instruction::GLOBAL_TO_MARKET_DEPOSIT);
    data.extend_from_slice(&params.amount.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build InitPositionTokens instruction.
///
/// Create position, all conditional token ATAs, and an Address Lookup Table.
///
/// Accounts (12 + num_outcomes*2):
/// 0. user (signer, mut)
/// 1. exchange (readonly)
/// 2. market (readonly)
/// 3. deposit_mint (readonly)
/// 4. vault (readonly)
/// 5. position (mut)
/// 6. lookup_table (mut)
/// 7. mint_authority (readonly)
/// 8. token_2022_program (readonly)
/// 9. ata_program (readonly)
/// 10. alt_program (readonly)
/// 11. system_program (readonly)
/// + per outcome: conditional_mint[i] (readonly), position_conditional_ata[i] (mut)
pub fn build_init_position_tokens_ix(
    params: &InitPositionTokensParams,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (vault, _) = get_vault_pda(&params.deposit_mint, &params.market, program_id);
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);
    let (lookup_table, _) = get_position_alt_pda(&position, params.recent_slot);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);

    let mut keys = vec![
        signer_mut(params.user),
        readonly(exchange),
        readonly(params.market),
        readonly(params.deposit_mint),
        readonly(vault),
        writable(position),
        writable(lookup_table),
        readonly(mint_authority),
        readonly(TOKEN_2022_PROGRAM_ID),
        readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
        readonly(*ALT_PROGRAM_ID),
        readonly(system_program_id()),
    ];

    for i in 0..num_outcomes {
        let (mint, _) =
            get_conditional_mint_pda(&params.market, &params.deposit_mint, i, program_id);
        keys.push(readonly(mint));
        let position_ata = get_conditional_token_ata(&position, &mint);
        keys.push(writable(position_ata));
    }

    let mut data = Vec::with_capacity(9);
    data.push(instruction::INIT_POSITION_TOKENS);
    data.extend_from_slice(&params.recent_slot.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build DepositAndSwap instruction.
///
/// Atomic deposit from global + mint conditional tokens + swap.
/// Data format is identical to MatchOrdersMulti.
///
/// Account layout:
///   Fixed (8): operator, exchange, market, deposit_mint, vault, global_deposit_token,
///              mint_authority, token_program
///   Taker block (6 or 7): [order_status], nonce, position, base_mint, quote_mint,
///                          token_2022_program, system_program
///   Conditional mints + taker ATAs (num_outcomes * 2)
///   Per-maker blocks: [order_status], nonce, position, user_global_deposit, ata[0..num_outcomes]
pub fn build_deposit_and_swap_ix(
    params: &DepositAndSwapParams,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.maker_orders.is_empty() {
        return Err(SdkError::MissingField("maker_orders".to_string()));
    }
    if params.maker_orders.len() > MAX_MAKERS {
        return Err(SdkError::TooManyMakers { count: params.maker_orders.len() });
    }
    if params.maker_orders.len() != params.maker_fill_amounts.len() {
        return Err(SdkError::MissingField("maker_fill_amounts".to_string()));
    }
    if params.maker_orders.len() != params.taker_fill_amounts.len() {
        return Err(SdkError::MissingField("taker_fill_amounts".to_string()));
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let (vault, _) = get_vault_pda(&params.deposit_mint, &params.market, program_id);
    let (global_deposit_token, _) =
        get_global_deposit_token_pda(&params.deposit_mint, program_id);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);

    let taker_order_hash = params.taker_order.hash();
    let (taker_nonce, _) = get_user_nonce_pda(&params.taker_order.maker, program_id);
    let (taker_position, _) =
        get_position_pda(&params.taker_order.maker, &params.market, program_id);

    let taker_full_fill = (params.full_fill_bitmask >> 7) & 1 == 1;

    let mut keys = Vec::new();

    // Fixed accounts (8)
    keys.push(signer_mut(params.operator));
    keys.push(readonly(exchange));
    keys.push(readonly(params.market));
    keys.push(readonly(params.deposit_mint));
    keys.push(writable(vault));
    keys.push(readonly(global_deposit_token));
    keys.push(readonly(mint_authority));
    keys.push(readonly(TOKEN_PROGRAM_ID));

    // Taker block
    if !taker_full_fill {
        let (taker_order_status, _) = get_order_status_pda(&taker_order_hash, program_id);
        keys.push(writable(taker_order_status));
    }
    keys.push(readonly(taker_nonce));
    keys.push(writable(taker_position));
    keys.push(readonly(params.base_mint));
    keys.push(readonly(params.quote_mint));
    keys.push(readonly(TOKEN_2022_PROGRAM_ID));
    keys.push(readonly(system_program_id()));

    // Conditional mints (writable - program mints tokens) + taker position ATAs (num_outcomes * 2)
    for i in 0..num_outcomes {
        let (cond_mint, _) =
            get_conditional_mint_pda(&params.market, &params.deposit_mint, i, program_id);
        keys.push(writable(cond_mint));
        let taker_ata = get_conditional_token_ata(&taker_position, &cond_mint);
        keys.push(writable(taker_ata));
    }

    // Per-maker blocks
    for (i, maker_order) in params.maker_orders.iter().enumerate() {
        let maker_full_fill = (params.full_fill_bitmask >> i) & 1 == 1;

        if !maker_full_fill {
            let maker_order_hash = maker_order.hash();
            let (maker_order_status, _) = get_order_status_pda(&maker_order_hash, program_id);
            keys.push(writable(maker_order_status));
        }

        let (maker_nonce, _) = get_user_nonce_pda(&maker_order.maker, program_id);
        let (maker_position, _) =
            get_position_pda(&maker_order.maker, &params.market, program_id);
        let (maker_global_deposit, _) =
            get_user_global_deposit_pda(&maker_order.maker, &params.deposit_mint, program_id);

        keys.push(readonly(maker_nonce));
        keys.push(writable(maker_position));
        keys.push(writable(maker_global_deposit));

        // Maker's conditional token ATAs for each outcome
        for j in 0..num_outcomes {
            let (cond_mint, _) =
                get_conditional_mint_pda(&params.market, &params.deposit_mint, j, program_id);
            let maker_ata = get_conditional_token_ata(&maker_position, &cond_mint);
            keys.push(writable(maker_ata));
        }
    }

    // Build data - same format as MatchOrdersMulti
    let taker_compact = params.taker_order.to_order();
    let num_makers = params.maker_orders.len() as u8;

    let data_size = 1 + 29 + 64 + 1 + 1 + (params.maker_orders.len() * 109);
    let mut data = Vec::with_capacity(data_size);

    data.push(instruction::DEPOSIT_AND_SWAP);
    data.extend_from_slice(&taker_compact.serialize());
    data.extend_from_slice(&params.taker_order.signature);
    data.push(num_makers);
    data.push(params.full_fill_bitmask);

    for (i, maker_order) in params.maker_orders.iter().enumerate() {
        let maker_compact = maker_order.to_order();
        data.extend_from_slice(&maker_compact.serialize());
        data.extend_from_slice(&maker_order.signature);
        data.extend_from_slice(&params.maker_fill_amounts[i].to_le_bytes());
        data.extend_from_slice(&params.taker_fill_amounts[i].to_le_bytes());
    }

    Ok(Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::constants::PROGRAM_ID;

    fn test_program_id() -> Pubkey {
        *PROGRAM_ID
    }

    #[test]
    fn test_build_initialize_ix() {
        let authority = Pubkey::new_unique();
        let program_id = test_program_id();

        let ix = build_initialize_ix(&authority, &program_id);

        assert_eq!(ix.program_id, program_id);
        assert_eq!(ix.accounts.len(), 3);
        assert_eq!(ix.data, vec![instruction::INITIALIZE]);
    }

    #[test]
    fn test_build_increment_nonce_ix() {
        let user = Pubkey::new_unique();
        let program_id = test_program_id();

        let ix = build_increment_nonce_ix(&user, &program_id);

        assert_eq!(ix.program_id, program_id);
        assert_eq!(ix.accounts.len(), 3);
        assert_eq!(ix.data, vec![instruction::INCREMENT_NONCE]);
    }

    #[test]
    fn test_build_set_paused_ix() {
        let authority = Pubkey::new_unique();
        let program_id = test_program_id();

        let ix_pause = build_set_paused_ix(&authority, true, &program_id);
        assert_eq!(ix_pause.data, vec![instruction::SET_PAUSED, 1]);

        let ix_unpause = build_set_paused_ix(&authority, false, &program_id);
        assert_eq!(ix_unpause.data, vec![instruction::SET_PAUSED, 0]);
    }

    #[test]
    fn test_build_set_operator_ix() {
        let authority = Pubkey::new_unique();
        let new_operator = Pubkey::new_unique();
        let program_id = test_program_id();

        let ix = build_set_operator_ix(&authority, &new_operator, &program_id);

        assert_eq!(ix.data.len(), 33);
        assert_eq!(ix.data[0], instruction::SET_OPERATOR);
        assert_eq!(&ix.data[1..33], new_operator.as_ref());
    }

    #[test]
    fn test_build_create_market_ix() {
        let params = CreateMarketParams {
            authority: Pubkey::new_unique(),
            num_outcomes: 3,
            oracle: Pubkey::new_unique(),
            question_id: [42u8; 32],
        };
        let program_id = test_program_id();

        let ix = build_create_market_ix(&params, 0, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 4);
        assert_eq!(ix.data.len(), 66); // 1 + 1 + 32 + 32
        assert_eq!(ix.data[0], instruction::CREATE_MARKET);
        assert_eq!(ix.data[1], 3);
    }

    #[test]
    fn test_build_create_market_invalid_outcomes() {
        let params = CreateMarketParams {
            authority: Pubkey::new_unique(),
            num_outcomes: 7, // Invalid - max is 6
            oracle: Pubkey::new_unique(),
            question_id: [0u8; 32],
        };
        let program_id = test_program_id();

        let result = build_create_market_ix(&params, 0, &program_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_activate_market_ix() {
        let params = ActivateMarketParams {
            authority: Pubkey::new_unique(),
            market_id: 5,
        };
        let program_id = test_program_id();

        let ix = build_activate_market_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 3);
        assert_eq!(ix.data, vec![instruction::ACTIVATE_MARKET]);
    }

    #[test]
    fn test_build_settle_market_ix() {
        let params = SettleMarketParams {
            oracle: Pubkey::new_unique(),
            market_id: 1,
            winning_outcome: 2,
        };
        let program_id = test_program_id();

        let ix = build_settle_market_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 3);
        assert_eq!(ix.data.len(), 2);
        assert_eq!(ix.data[0], instruction::SETTLE_MARKET);
        assert_eq!(ix.data[1], 2);
    }

    #[test]
    fn test_build_cancel_order_ix() {
        let maker = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let program_id = test_program_id();

        let order = SignedOrder {
            nonce: 1,
            maker,
            market,
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: crate::program::types::OrderSide::Bid,
            maker_amount: 100,
            taker_amount: 50,
            expiration: 0,
            signature: [0u8; 64],
        };

        let ix = build_cancel_order_ix(&maker, &market, &order, &program_id);

        assert_eq!(ix.accounts.len(), 4);
        assert_eq!(ix.data.len(), 258); // 1 + 32 + 225
        assert_eq!(ix.data[0], instruction::CANCEL_ORDER);
    }

    #[test]
    fn test_build_withdraw_from_position_ix() {
        let program_id = test_program_id();
        let params = WithdrawFromPositionParams {
            user: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            amount: 1000,
            outcome_index: 0,
        };

        let ix = build_withdraw_from_position_ix(&params, true, &program_id);

        assert_eq!(ix.accounts.len(), 7);
        assert_eq!(ix.data.len(), 10); // 1 + 8 + 1
        assert_eq!(ix.data[0], instruction::WITHDRAW_FROM_POSITION);
        assert_eq!(ix.data[9], 0); // outcome_index
    }

    #[test]
    fn test_build_create_orderbook_ix() {
        let program_id = test_program_id();
        let params = CreateOrderbookParams {
            payer: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            mint_a: Pubkey::new_unique(),
            mint_b: Pubkey::new_unique(),
            recent_slot: 12345,
        };

        let ix = build_create_orderbook_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 9);
        assert_eq!(ix.data.len(), 9); // 1 + 8
        assert_eq!(ix.data[0], instruction::CREATE_ORDERBOOK);
    }

    #[test]
    fn test_build_set_authority_ix() {
        let program_id = test_program_id();
        let params = SetAuthorityParams {
            current_authority: Pubkey::new_unique(),
            new_authority: Pubkey::new_unique(),
        };

        let ix = build_set_authority_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 2);
        assert_eq!(ix.data.len(), 33); // 1 + 32
        assert_eq!(ix.data[0], instruction::SET_AUTHORITY);
        assert_eq!(&ix.data[1..33], params.new_authority.as_ref());
    }

    #[test]
    fn test_build_match_orders_multi_ix_data_format() {
        let program_id = test_program_id();
        let operator = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let taker = SignedOrder {
            nonce: 1,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: crate::program::types::OrderSide::Bid,
            maker_amount: 100,
            taker_amount: 50,
            expiration: 0,
            signature: [1u8; 64],
        };

        let maker = SignedOrder {
            nonce: 2,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: crate::program::types::OrderSide::Ask,
            maker_amount: 50,
            taker_amount: 100,
            expiration: 0,
            signature: [2u8; 64],
        };

        let params = MatchOrdersMultiParams {
            operator,
            market,
            base_mint,
            quote_mint,
            taker_order: taker,
            maker_orders: vec![maker],
            maker_fill_amounts: vec![50],
            taker_fill_amounts: vec![100],
            full_fill_bitmask: 0,
        };

        let ix = build_match_orders_multi_ix(&params, &program_id).unwrap();

        // Data: 1 + 29 + 64 + 1 + 1 + 109 = 205
        assert_eq!(ix.data.len(), 205);
        assert_eq!(ix.data[0], instruction::MATCH_ORDERS_MULTI);

        // With bitmask=0 (no full fills):
        // Taker: 12 accounts, Maker: 5 accounts = 17 total
        assert_eq!(ix.accounts.len(), 17);
    }

    #[test]
    fn test_build_match_orders_multi_ix_full_fill() {
        let program_id = test_program_id();
        let operator = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let taker = SignedOrder {
            nonce: 1,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: crate::program::types::OrderSide::Bid,
            maker_amount: 100,
            taker_amount: 50,
            expiration: 0,
            signature: [1u8; 64],
        };

        let maker = SignedOrder {
            nonce: 2,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: crate::program::types::OrderSide::Ask,
            maker_amount: 50,
            taker_amount: 100,
            expiration: 0,
            signature: [2u8; 64],
        };

        // bit 0 = 1 (maker 0 full fill), bit 7 = 1 (taker full fill)
        let params = MatchOrdersMultiParams {
            operator,
            market,
            base_mint,
            quote_mint,
            taker_order: taker,
            maker_orders: vec![maker],
            maker_fill_amounts: vec![50],
            taker_fill_amounts: vec![100],
            full_fill_bitmask: 0b10000001,
        };

        let ix = build_match_orders_multi_ix(&params, &program_id).unwrap();

        // With bitmask=0x81 (taker + maker 0 full fill):
        // Taker: 11 accounts (no order_status), Maker: 4 accounts (no order_status) = 15 total
        assert_eq!(ix.accounts.len(), 15);
    }

    #[test]
    fn test_build_whitelist_deposit_token_ix() {
        let program_id = test_program_id();
        let params = WhitelistDepositTokenParams {
            authority: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
        };

        let ix = build_whitelist_deposit_token_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 5);
        assert_eq!(ix.data, vec![instruction::WHITELIST_DEPOSIT_TOKEN]);
    }

    #[test]
    fn test_build_deposit_to_global_ix() {
        let program_id = test_program_id();
        let params = DepositToGlobalParams {
            user: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            amount: 1_000_000,
        };

        let ix = build_deposit_to_global_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 7);
        assert_eq!(ix.data.len(), 9);
        assert_eq!(ix.data[0], instruction::DEPOSIT_TO_GLOBAL);
    }

    #[test]
    fn test_build_global_to_market_deposit_ix() {
        let program_id = test_program_id();
        let params = GlobalToMarketDepositParams {
            user: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            deposit_mint: Pubkey::new_unique(),
            amount: 500_000,
        };

        let ix = build_global_to_market_deposit_ix(&params, 3, &program_id);

        // 14 fixed + 3*2 conditional = 20
        assert_eq!(ix.accounts.len(), 20);
        assert_eq!(ix.data.len(), 9);
        assert_eq!(ix.data[0], instruction::GLOBAL_TO_MARKET_DEPOSIT);
    }

    #[test]
    fn test_build_init_position_tokens_ix() {
        let program_id = test_program_id();
        let params = InitPositionTokensParams {
            user: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            deposit_mint: Pubkey::new_unique(),
            recent_slot: 12345,
        };

        let ix = build_init_position_tokens_ix(&params, 3, &program_id);

        // 12 fixed + 3*2 conditional = 18
        assert_eq!(ix.accounts.len(), 18);
        assert_eq!(ix.data.len(), 9);
        assert_eq!(ix.data[0], instruction::INIT_POSITION_TOKENS);
    }

    #[test]
    fn test_build_deposit_and_swap_ix() {
        let program_id = test_program_id();
        let market = Pubkey::new_unique();
        let deposit_mint = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let taker = SignedOrder {
            nonce: 1,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: crate::program::types::OrderSide::Bid,
            maker_amount: 100,
            taker_amount: 50,
            expiration: 0,
            signature: [1u8; 64],
        };

        let maker = SignedOrder {
            nonce: 2,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: crate::program::types::OrderSide::Ask,
            maker_amount: 50,
            taker_amount: 100,
            expiration: 0,
            signature: [2u8; 64],
        };

        let params = DepositAndSwapParams {
            operator: Pubkey::new_unique(),
            market,
            deposit_mint,
            base_mint,
            quote_mint,
            taker_order: taker,
            maker_orders: vec![maker],
            maker_fill_amounts: vec![50],
            taker_fill_amounts: vec![100],
            full_fill_bitmask: 0,
        };

        let num_outcomes = 3u8;
        let ix = build_deposit_and_swap_ix(&params, num_outcomes, &program_id).unwrap();

        // Data: 1 + 29 + 64 + 1 + 1 + 109 = 205
        assert_eq!(ix.data.len(), 205);
        assert_eq!(ix.data[0], instruction::DEPOSIT_AND_SWAP);

        // Account counts with bitmask=0 (all need order_status):
        // Fixed: 8
        // Taker block: 7 (with order_status)
        // Conditional: 3*2 = 6
        // Maker block: 1*(1+3+3) = 7
        // Total: 8 + 7 + 6 + 7 = 28
        assert_eq!(ix.accounts.len(), 28);
    }
}
