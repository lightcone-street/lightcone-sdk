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
    instruction, ALT_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, MAX_MAKERS, MAX_OUTCOMES,
    MIN_OUTCOMES, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
};
use crate::program::error::{SdkError, SdkResult};
use crate::program::orders::OrderPayload;
use crate::program::pda::{
    get_alt_pda, get_condition_tombstone_pda, get_conditional_mint_pda, get_exchange_pda,
    get_global_deposit_token_pda, get_market_pda, get_mint_authority_pda, get_order_status_pda,
    get_orderbook_pda, get_position_alt_pda, get_position_pda, get_user_global_deposit_pda,
    get_user_nonce_pda, get_vault_pda,
};
use crate::program::types::{
    ActivateMarketParams, AddDepositMintParams, BuildDepositParams, BuildMergeParams,
    CreateMarketParams, CreateOrderbookParams, DepositAndSwapParams, DepositToGlobalAltContext,
    DepositToGlobalParams, ExtendPositionTokensParams, GlobalToMarketDepositParams,
    InitPositionTokensParams, MatchOrdersMultiParams, RedeemWinningsParams, SetAuthorityParams,
    SetManagerParams, SettleMarketParams, WhitelistDepositTokenParams, WithdrawFromGlobalParams,
    WithdrawFromPositionParams,
};
use crate::program::utils::{
    get_conditional_token_ata, get_deposit_token_ata, serialize_outcome_metadata,
    validate_outcome_count, OutcomeMetadataInput,
};
use crate::program::{derive_condition_id, ORDER_SIZE, SIGNATURE_SIZE};

// ============================================================================
// Helper Functions
// ============================================================================

const MATCH_ORDER_HEADER_SIZE: usize = ORDER_SIZE + SIGNATURE_SIZE + 2;
const DEPOSIT_AND_SWAP_HEADER_SIZE: usize = ORDER_SIZE + SIGNATURE_SIZE + 3;
const MAKER_MATCH_SIZE: usize = ORDER_SIZE + SIGNATURE_SIZE + 16;

/// Create an account meta for a signer+writable account.
fn signer_mut(pubkey: Pubkey) -> AccountMeta {
    AccountMeta::new(pubkey, true)
}

/// Create an account meta for a read-only signer account.
fn signer(pubkey: Pubkey) -> AccountMeta {
    AccountMeta::new_readonly(pubkey, true)
}

/// Create an account meta for a writable account.
fn writable(pubkey: Pubkey) -> AccountMeta {
    AccountMeta::new(pubkey, false)
}

/// Create an account meta for a read-only account.
fn readonly(pubkey: Pubkey) -> AccountMeta {
    AccountMeta::new_readonly(pubkey, false)
}

struct OrderbookMintInput {
    mint: Pubkey,
    deposit_mint: Pubkey,
    outcome_index: u8,
    is_base: bool,
}

struct CanonicalOrderbookMints {
    mint_a: OrderbookMintInput,
    mint_b: OrderbookMintInput,
}

impl CanonicalOrderbookMints {
    fn from_params(params: &CreateOrderbookParams) -> SdkResult<Self> {
        if params.base_index > 1 {
            return Err(SdkError::InvalidOutcomeIndex {
                index: params.base_index,
                max: 1,
            });
        }
        if params.mint_a == params.mint_b {
            return Err(SdkError::InvalidMintOrder);
        }

        let left = OrderbookMintInput {
            mint: params.mint_a,
            deposit_mint: params.mint_a_deposit_mint,
            outcome_index: params.mint_a_outcome_index,
            is_base: params.base_index == 0,
        };
        let right = OrderbookMintInput {
            mint: params.mint_b,
            deposit_mint: params.mint_b_deposit_mint,
            outcome_index: params.mint_b_outcome_index,
            is_base: params.base_index == 1,
        };

        let (mint_a, mint_b) = if left.mint.as_ref() < right.mint.as_ref() {
            (left, right)
        } else {
            (right, left)
        };

        Ok(Self { mint_a, mint_b })
    }

    fn base_index(&self) -> u8 {
        if self.mint_a.is_base {
            0
        } else {
            1
        }
    }
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
/// 0. manager (signer, mut) - Must be exchange manager
/// 1. exchange (mut) - Exchange PDA
/// 2. market (mut) - Market PDA
/// 3. system_program (readonly)
/// 4. condition_tombstone (mut) - Condition uniqueness PDA
pub fn build_create_market_ix(
    params: &CreateMarketParams,
    market_id: u64,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    validate_outcome_count(params.num_outcomes)?;

    let (exchange, _) = get_exchange_pda(program_id);
    let (market, _) = get_market_pda(market_id, program_id);
    let condition_id =
        derive_condition_id(&params.oracle, &params.question_id, params.num_outcomes);
    let (condition_tombstone, _) = get_condition_tombstone_pda(&condition_id, program_id);

    let keys = vec![
        signer_mut(params.manager),
        writable(exchange),
        writable(market),
        readonly(system_program_id()),
        writable(condition_tombstone),
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
/// Manager-only — must be called by the exchange manager.
///
/// Accounts:
/// 0. manager (signer, mut) - Must be exchange manager
/// 1. exchange (readonly) - Exchange PDA
/// 2. market (readonly)
/// 3. deposit_mint (readonly)
/// 4. vault (mut)
/// 5. mint_authority (readonly)
/// 6. token_program (SPL Token)
/// 7. token_2022_program
/// 8. system_program
/// 9. global_deposit_token
/// 10+ conditional_mints\[0..num_outcomes\]
pub fn build_add_deposit_mint_ix(
    params: &AddDepositMintParams,
    market: &Pubkey,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.outcome_metadata.len() != num_outcomes as usize {
        return Err(SdkError::InvalidOutcomeCount {
            count: params.outcome_metadata.len() as u8,
        });
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let (vault, _) = get_vault_pda(&params.deposit_mint, market, program_id);
    let (mint_authority, _) = get_mint_authority_pda(market, program_id);
    let (global_deposit_token, _) = get_global_deposit_token_pda(&params.deposit_mint, program_id);

    let mut keys = vec![
        signer_mut(params.manager),
        readonly(exchange),
        readonly(*market),
        readonly(params.deposit_mint),
        writable(vault),
        readonly(mint_authority),
        readonly(TOKEN_PROGRAM_ID),
        readonly(TOKEN_2022_PROGRAM_ID),
        readonly(system_program_id()),
        readonly(global_deposit_token),
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

/// Build Deposit (MintCompleteSet) instruction.
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
pub fn build_deposit_ix(
    params: &BuildDepositParams,
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

/// Build Merge (MergeCompleteSet) instruction.
///
/// Burns all outcome tokens from Position and releases collateral.
pub fn build_merge_ix(
    params: &BuildMergeParams,
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
/// Marks an existing on-chain order status as cancelled and closes it.
///
/// Accounts:
/// 0. operator (signer, mut)
/// 1. exchange (readonly)
/// 2. market (readonly)
/// 3. order_status (mut)
pub fn build_cancel_order_ix(
    operator: &Pubkey,
    market: &Pubkey,
    order: &OrderPayload,
    program_id: &Pubkey,
) -> Instruction {
    let order_hash = order.hash();
    let (exchange, _) = get_exchange_pda(program_id);
    let (order_status, _) = get_order_status_pda(&order_hash, program_id);

    let keys = vec![
        signer_mut(*operator),
        readonly(exchange),
        readonly(*market),
        writable(order_status),
    ];

    // Data: [discriminator(1), order_hash(32), OrderPayload(233)] = 266 bytes
    let mut data = Vec::with_capacity(266);
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
/// Oracle resolves the market with payout numerators. The program computes the
/// denominator as the checked sum of the submitted numerators.
pub fn build_settle_market_ix(
    params: &SettleMarketParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    validate_payout_numerators(&params.payout_numerators)?;

    let (exchange, _) = get_exchange_pda(program_id);
    let (market, _) = get_market_pda(params.market_id, program_id);

    let keys = vec![signer(params.oracle), readonly(exchange), writable(market)];

    let mut data = Vec::with_capacity(1 + (params.payout_numerators.len() * 4));
    data.push(instruction::SETTLE_MARKET);
    for numerator in &params.payout_numerators {
        data.extend_from_slice(&numerator.to_le_bytes());
    }

    Ok(Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    })
}

fn validate_payout_numerators(payout_numerators: &[u32]) -> SdkResult<()> {
    let count = payout_numerators.len();
    if count < MIN_OUTCOMES as usize || count > MAX_OUTCOMES as usize {
        return Err(SdkError::InvalidOutcomeCount {
            count: u8::try_from(count).unwrap_or(u8::MAX),
        });
    }

    let mut denominator = 0u32;
    for numerator in payout_numerators {
        denominator = denominator
            .checked_add(*numerator)
            .ok_or(SdkError::Overflow)?;
    }

    if denominator == 0 {
        return Err(SdkError::InvalidPayoutNumerators);
    }

    Ok(())
}

/// Build RedeemWinnings instruction.
///
/// Redeem winning outcome tokens from Position for collateral.
pub fn build_redeem_winnings_ix(
    params: &RedeemWinningsParams,
    outcome_index: u8,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (vault, _) = get_vault_pda(&params.deposit_mint, &params.market, program_id);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);
    let (conditional_mint, _) = get_conditional_mint_pda(
        &params.market,
        &params.deposit_mint,
        outcome_index,
        program_id,
    );
    let position_conditional_ata = get_conditional_token_ata(&position, &conditional_mint);
    let user_deposit_ata = get_deposit_token_ata(&params.user, &params.deposit_mint);

    let keys = vec![
        signer_mut(params.user),
        readonly(params.market),
        readonly(params.deposit_mint),
        writable(vault),
        writable(conditional_mint),
        readonly(position),
        writable(position_conditional_ata),
        writable(user_deposit_ata),
        readonly(mint_authority),
        readonly(TOKEN_PROGRAM_ID),
        readonly(TOKEN_2022_PROGRAM_ID),
        readonly(exchange),
    ];

    let mut data = Vec::with_capacity(10);
    data.push(instruction::REDEEM_WINNINGS);
    data.extend_from_slice(&params.amount.to_le_bytes());
    data.push(outcome_index);

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build SetPaused instruction.
///
/// Admin: pause/unpause exchange.
pub fn build_set_paused_ix(authority: &Pubkey, paused: bool, program_id: &Pubkey) -> Instruction {
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
/// Manager: Pending → Active.
pub fn build_activate_market_ix(params: &ActivateMarketParams, program_id: &Pubkey) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (market, _) = get_market_pda(params.market_id, program_id);

    let keys = vec![
        signer_mut(params.manager),
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
/// Match taker against makers.
///
/// Data format:
/// [0]       discriminator
/// [1..38]   taker Order (37 bytes)
/// [38..102] taker_signature (64 bytes)
/// [102]     num_makers
/// [103]     full_fill_bitmask
/// Per maker (117 bytes each):
///   [+0..+37]    maker Order (37)
///   [+37..+101]  maker_signature (64)
///   [+101..+109] maker_fill_amount (8)
///   [+109..+117] taker_fill_amount (8)
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
        return Err(SdkError::TooManyMakers {
            count: params.maker_orders.len(),
        });
    }
    if params.maker_orders.len() != params.maker_fill_amounts.len() {
        return Err(SdkError::MissingField("maker_fill_amounts".to_string()));
    }
    if params.maker_orders.len() != params.taker_fill_amounts.len() {
        return Err(SdkError::MissingField("taker_fill_amounts".to_string()));
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let (orderbook, _) = get_orderbook_pda(&params.base_mint, &params.quote_mint, program_id);
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
    keys.push(readonly(orderbook));

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

    let data_size = 1 + MATCH_ORDER_HEADER_SIZE + (params.maker_orders.len() * MAKER_MATCH_SIZE);
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
/// Manager-only — must be called by the exchange manager.
///
/// Accounts (11):
/// 0. manager (signer, mut) - Must be exchange manager
/// 1. market (readonly)
/// 2. mint_a (readonly, canonical order)
/// 3. mint_b (readonly, canonical order)
/// 4. orderbook (mut)
/// 5. lookup_table (mut)
/// 6. exchange (readonly)
/// 7. alt_program (readonly)
/// 8. system_program (readonly)
/// 9. mint_a_deposit_mint
/// 10. mint_b_deposit_mint
pub fn build_create_orderbook_ix(
    params: &CreateOrderbookParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    let canonical = CanonicalOrderbookMints::from_params(params)?;
    let (exchange, _) = get_exchange_pda(program_id);
    let (orderbook, _) =
        get_orderbook_pda(&canonical.mint_a.mint, &canonical.mint_b.mint, program_id);
    let (lookup_table, _) = get_alt_pda(&orderbook, params.recent_slot);

    let keys = vec![
        signer_mut(params.manager),
        readonly(params.market),
        readonly(canonical.mint_a.mint),
        readonly(canonical.mint_b.mint),
        writable(orderbook),
        writable(lookup_table),
        readonly(exchange),
        readonly(*ALT_PROGRAM_ID),
        readonly(system_program_id()),
        readonly(canonical.mint_a.deposit_mint),
        readonly(canonical.mint_b.deposit_mint),
    ];

    // Data: [discriminator(1), recent_slot(8), base_index(1), mint_a_outcome_index(1), mint_b_outcome_index(1)] = 12 bytes
    let mut data = Vec::with_capacity(12);
    data.push(instruction::CREATE_ORDERBOOK);
    data.extend_from_slice(&params.recent_slot.to_le_bytes());
    data.push(canonical.base_index());
    data.push(canonical.mint_a.outcome_index);
    data.push(canonical.mint_b.outcome_index);

    Ok(Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    })
}

/// Build SetAuthority instruction.
///
/// Change the exchange authority.
///
/// Accounts (2):
/// 0. authority (signer)
/// 1. exchange (mut)
pub fn build_set_authority_ix(params: &SetAuthorityParams, program_id: &Pubkey) -> Instruction {
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

/// Build SetManager instruction.
///
/// Change the exchange manager.
///
/// Accounts (2):
/// 0. authority (signer)
/// 1. exchange (mut)
pub fn build_set_manager_ix(params: &SetManagerParams, program_id: &Pubkey) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![signer_mut(params.authority), writable(exchange)];

    let mut data = Vec::with_capacity(33);
    data.push(instruction::SET_MANAGER);
    data.extend_from_slice(params.new_manager.as_ref());

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
        writable(exchange),
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
/// Accounts (8):
/// 0. user (signer, mut)
/// 1. global_deposit_token (readonly) - Whitelist PDA
/// 2. mint (readonly)
/// 3. user_global_deposit (mut) - User's deposit PDA
/// 4. user_token_account (mut) - User's source token account
/// 5. token_program (readonly)
/// 6. system_program (readonly)
/// 7. exchange (readonly) - Exchange PDA for pause validation
pub fn build_deposit_to_global_ix(
    params: &DepositToGlobalParams,
    program_id: &Pubkey,
) -> Instruction {
    build_deposit_to_global_ix_inner(params, None, program_id)
}

/// Build DepositToGlobal instruction with user deposit ALT create/extend accounts.
pub fn build_deposit_to_global_ix_with_alt(
    params: &DepositToGlobalParams,
    alt_context: DepositToGlobalAltContext,
    program_id: &Pubkey,
) -> Instruction {
    build_deposit_to_global_ix_inner(params, Some(alt_context), program_id)
}

fn build_deposit_to_global_ix_inner(
    params: &DepositToGlobalParams,
    alt_context: Option<DepositToGlobalAltContext>,
    program_id: &Pubkey,
) -> Instruction {
    let (global_deposit_token, _) = get_global_deposit_token_pda(&params.mint, program_id);
    let (user_global_deposit, _) =
        get_user_global_deposit_pda(&params.user, &params.mint, program_id);
    let (exchange, _) = get_exchange_pda(program_id);
    let user_token_account = get_deposit_token_ata(&params.user, &params.mint);

    let mut keys = vec![
        signer_mut(params.user),
        readonly(global_deposit_token),
        readonly(params.mint),
        writable(user_global_deposit),
        writable(user_token_account),
        readonly(TOKEN_PROGRAM_ID),
        readonly(system_program_id()),
        readonly(exchange),
    ];

    let mut data = Vec::with_capacity(9);
    data.push(instruction::DEPOSIT_TO_GLOBAL);
    data.extend_from_slice(&params.amount.to_le_bytes());

    if let Some(alt_context) = alt_context {
        let (user_nonce, _) = get_user_nonce_pda(&params.user, program_id);
        let lookup_table = match alt_context {
            DepositToGlobalAltContext::Create { recent_slot } => {
                data.extend_from_slice(&recent_slot.to_le_bytes());
                get_alt_pda(&user_nonce, recent_slot).0
            }
            DepositToGlobalAltContext::Extend { lookup_table } => lookup_table,
        };

        keys.push(readonly(user_nonce));
        keys.push(writable(lookup_table));
        keys.push(readonly(*ALT_PROGRAM_ID));
    }

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
    let (global_deposit_token, _) = get_global_deposit_token_pda(&params.deposit_mint, program_id);
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
/// Permissionless — anyone (e.g., backend operator) can pay.
///
/// Accounts (11 + per deposit_mint: 3 + num_outcomes*2):
/// 0. payer (signer, mut) - Pays for account creation
/// 1. user (readonly) - Position owner
/// 2. exchange (readonly)
/// 3. market (readonly)
/// 4. position (mut)
/// 5. lookup_table (mut)
/// 6. mint_authority (readonly)
/// 7. token_2022_program (readonly)
/// 8. ata_program (readonly)
/// 9. alt_program (readonly)
/// 10. system_program (readonly)
/// + per deposit_mint: deposit_mint, vault, gdt, [cond_mint, ata] × num_outcomes
pub fn build_init_position_tokens_ix(
    params: &InitPositionTokensParams,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);
    let (lookup_table, _) = get_position_alt_pda(&position, params.recent_slot);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);

    let mut keys = vec![
        signer_mut(params.payer),
        readonly(params.user),
        readonly(exchange),
        readonly(params.market),
        writable(position),
        writable(lookup_table),
        readonly(mint_authority),
        readonly(TOKEN_2022_PROGRAM_ID),
        readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
        readonly(*ALT_PROGRAM_ID),
        readonly(system_program_id()),
    ];

    for deposit_mint in &params.deposit_mints {
        let (vault, _) = get_vault_pda(deposit_mint, &params.market, program_id);
        let (gdt, _) = get_global_deposit_token_pda(deposit_mint, program_id);
        keys.push(readonly(*deposit_mint));
        keys.push(readonly(vault));
        keys.push(readonly(gdt));

        for i in 0..num_outcomes {
            let (mint, _) = get_conditional_mint_pda(&params.market, deposit_mint, i, program_id);
            keys.push(readonly(mint));
            let position_ata = get_conditional_token_ata(&position, &mint);
            keys.push(writable(position_ata));
        }
    }

    let mut data = Vec::with_capacity(10);
    data.push(instruction::INIT_POSITION_TOKENS);
    data.extend_from_slice(&params.recent_slot.to_le_bytes());
    data.push(params.deposit_mints.len() as u8);

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

/// Build DepositAndSwap instruction.
///
/// Unified order execution: participants can deposit from global deposits and/or swap
/// conditional tokens in a single instruction. Each participant's deposit is conditional
/// on the deposit_bitmask.
///
/// Account layout:
///   Fixed (6): operator, exchange, market, orderbook, mint_authority, token_program
///   Taker block (8-9): [order_status], nonce, position, base_mint, quote_mint,
///                       taker_receive_ata, taker_give_ata, token_2022_program, system_program
///   Taker deposit block (optional): deposit_mint, vault, gdt, user_global_deposit,
///                                    [cond_mint, ata] × num_outcomes
///   Per-maker blocks: [order_status], nonce, position,
///                      [deposit block if depositing],
///                      maker_receive_ata, maker_give_ata
pub fn build_deposit_and_swap_ix(
    params: &DepositAndSwapParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.makers.is_empty() {
        return Err(SdkError::MissingField("makers".to_string()));
    }
    if params.makers.len() > MAX_MAKERS {
        return Err(SdkError::TooManyMakers {
            count: params.makers.len(),
        });
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let (orderbook, _) = get_orderbook_pda(&params.base_mint, &params.quote_mint, program_id);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);
    let (taker_position, _) =
        get_position_pda(&params.taker_order.maker, &params.market, program_id);
    let (taker_nonce, _) = get_user_nonce_pda(&params.taker_order.maker, program_id);

    let taker_side = params.taker_order.side as u8;
    let (receive_mint, give_mint) = if taker_side == 0 {
        (&params.base_mint, &params.quote_mint)
    } else {
        (&params.quote_mint, &params.base_mint)
    };

    // Build bitmasks
    let mut full_fill_bitmask: u8 = 0;
    let mut deposit_bitmask: u8 = 0;
    if params.taker_is_full_fill {
        full_fill_bitmask |= 0x80;
    }
    if params.taker_is_deposit {
        deposit_bitmask |= 0x80;
    }
    for (i, maker) in params.makers.iter().enumerate() {
        if maker.is_full_fill {
            full_fill_bitmask |= 1 << i;
        }
        if maker.is_deposit {
            deposit_bitmask |= 1 << i;
        }
    }

    let mut keys = Vec::new();

    // Fixed accounts (6)
    keys.push(signer_mut(params.operator));
    keys.push(readonly(exchange));
    keys.push(readonly(params.market));
    keys.push(readonly(orderbook));
    keys.push(readonly(mint_authority));
    keys.push(readonly(TOKEN_PROGRAM_ID));

    // Taker order_status (only if not full fill)
    if !params.taker_is_full_fill {
        let taker_order_hash = params.taker_order.hash();
        let (taker_order_status, _) = get_order_status_pda(&taker_order_hash, program_id);
        keys.push(writable(taker_order_status));
    }

    // Taker common block
    let taker_receive_ata = get_conditional_token_ata(&taker_position, receive_mint);
    let taker_give_ata = get_conditional_token_ata(&taker_position, give_mint);
    keys.push(readonly(taker_nonce));
    keys.push(writable(taker_position));
    keys.push(readonly(params.base_mint));
    keys.push(readonly(params.quote_mint));
    keys.push(writable(taker_receive_ata));
    keys.push(writable(taker_give_ata));
    keys.push(readonly(TOKEN_2022_PROGRAM_ID));
    keys.push(readonly(system_program_id()));

    // Taker deposit block (only if taker deposits)
    if params.taker_is_deposit {
        let dm = &params.taker_deposit_mint;
        let (vault, _) = get_vault_pda(dm, &params.market, program_id);
        let (gdt, _) = get_global_deposit_token_pda(dm, program_id);
        let (taker_global_deposit, _) =
            get_user_global_deposit_pda(&params.taker_order.maker, dm, program_id);
        keys.push(readonly(*dm));
        keys.push(writable(vault));
        keys.push(readonly(gdt));
        keys.push(writable(taker_global_deposit));

        for i in 0..params.num_outcomes {
            let (cond_mint, _) = get_conditional_mint_pda(&params.market, dm, i, program_id);
            let ata = get_conditional_token_ata(&taker_position, &cond_mint);
            keys.push(writable(cond_mint));
            keys.push(writable(ata));
        }
    }

    // Per-maker blocks
    for maker in &params.makers {
        let (maker_nonce, _) = get_user_nonce_pda(&maker.order.maker, program_id);
        let (maker_position, _) = get_position_pda(&maker.order.maker, &params.market, program_id);

        if !maker.is_full_fill {
            let maker_order_hash = maker.order.hash();
            let (maker_order_status, _) = get_order_status_pda(&maker_order_hash, program_id);
            keys.push(writable(maker_order_status));
        }

        keys.push(readonly(maker_nonce));
        keys.push(writable(maker_position));

        // Maker deposit block (only if maker deposits)
        if maker.is_deposit {
            let dm = &maker.deposit_mint;
            let (vault, _) = get_vault_pda(dm, &params.market, program_id);
            let (gdt, _) = get_global_deposit_token_pda(dm, program_id);
            let (maker_global_deposit, _) =
                get_user_global_deposit_pda(&maker.order.maker, dm, program_id);
            keys.push(readonly(*dm));
            keys.push(writable(vault));
            keys.push(readonly(gdt));
            keys.push(writable(maker_global_deposit));

            for j in 0..params.num_outcomes {
                let (cond_mint, _) = get_conditional_mint_pda(&params.market, dm, j, program_id);
                let maker_ata = get_conditional_token_ata(&maker_position, &cond_mint);
                keys.push(writable(cond_mint));
                keys.push(writable(maker_ata));
            }
        }

        // Swap ATAs (always present)
        let maker_receive_ata = get_conditional_token_ata(&maker_position, receive_mint);
        let maker_give_ata = get_conditional_token_ata(&maker_position, give_mint);
        keys.push(writable(maker_receive_ata));
        keys.push(writable(maker_give_ata));
    }

    // Build instruction data
    let taker_compact = params.taker_order.to_order();
    let num_makers = params.makers.len() as u8;

    let data_size = 1 + DEPOSIT_AND_SWAP_HEADER_SIZE + (params.makers.len() * MAKER_MATCH_SIZE);
    let mut data = Vec::with_capacity(data_size);

    data.push(instruction::DEPOSIT_AND_SWAP);
    data.extend_from_slice(&taker_compact.serialize());
    data.extend_from_slice(&params.taker_order.signature);
    data.push(num_makers);
    data.push(full_fill_bitmask);
    data.push(deposit_bitmask);

    for maker in &params.makers {
        let maker_compact = maker.order.to_order();
        data.extend_from_slice(&maker_compact.serialize());
        data.extend_from_slice(&maker.order.signature);
        data.extend_from_slice(&maker.maker_fill_amount.to_le_bytes());
        data.extend_from_slice(&maker.taker_fill_amount.to_le_bytes());
    }

    Ok(Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    })
}

/// Build ExtendPositionTokens instruction.
///
/// Extend an existing position ALT with entries for new deposit mints.
/// Operator-only — the exchange operator pays to extend ALTs for users.
///
/// Accounts (10 + per deposit_mint: 3 + num_outcomes*2):
/// 0. operator (signer, mut)
/// 1. user (readonly) - Position owner
/// 2. exchange (readonly)
/// 3. market (readonly)
/// 4. position (readonly) - Existing Position PDA
/// 5. lookup_table (mut) - Existing ALT (authority = position PDA)
/// 6. token_2022_program (readonly)
/// 7. ata_program (readonly)
/// 8. alt_program (readonly)
/// 9. system_program (readonly)
/// + per deposit_mint: deposit_mint, vault, global_deposit_token,
///   then per outcome: conditional_mint, position_conditional_ata
pub fn build_extend_position_tokens_ix(
    params: &ExtendPositionTokensParams,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.deposit_mints.is_empty() {
        return Err(SdkError::MissingField("deposit_mints".to_string()));
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);

    let mut keys = vec![
        signer_mut(params.operator),
        readonly(params.user),
        readonly(exchange),
        readonly(params.market),
        readonly(position),
        writable(params.lookup_table),
        readonly(TOKEN_2022_PROGRAM_ID),
        readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
        readonly(*ALT_PROGRAM_ID),
        readonly(system_program_id()),
    ];

    for deposit_mint in &params.deposit_mints {
        let (vault, _) = get_vault_pda(deposit_mint, &params.market, program_id);
        let (global_deposit_token, _) = get_global_deposit_token_pda(deposit_mint, program_id);

        keys.push(readonly(*deposit_mint));
        keys.push(readonly(vault));
        keys.push(readonly(global_deposit_token));

        for i in 0..num_outcomes {
            let (cond_mint, _) =
                get_conditional_mint_pda(&params.market, deposit_mint, i, program_id);
            let position_ata = get_conditional_token_ata(&position, &cond_mint);
            keys.push(readonly(cond_mint));
            keys.push(writable(position_ata));
        }
    }

    let data = vec![
        instruction::EXTEND_POSITION_TOKENS,
        params.deposit_mints.len() as u8,
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    })
}

// ============================================================================
// Withdraw From Global
// ============================================================================

/// Build a `withdraw_from_global` instruction.
///
/// Withdraws tokens from a user's global deposit account back to their wallet.
pub fn build_withdraw_from_global_ix(
    params: &WithdrawFromGlobalParams,
    program_id: &Pubkey,
) -> Instruction {
    let (global_deposit_token, _) = get_global_deposit_token_pda(&params.mint, program_id);
    let (user_global_deposit, _) =
        get_user_global_deposit_pda(&params.user, &params.mint, program_id);
    let (exchange, _) = get_exchange_pda(program_id);
    let user_token_account = get_deposit_token_ata(&params.user, &params.mint);

    let keys = vec![
        signer_mut(params.user),
        readonly(global_deposit_token),
        readonly(params.mint),
        writable(user_global_deposit),
        writable(user_token_account),
        readonly(TOKEN_PROGRAM_ID),
        readonly(exchange),
    ];

    let mut data = vec![instruction::WITHDRAW_FROM_GLOBAL];
    data.extend_from_slice(&params.amount.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: keys,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::LightconeEnv;
    use crate::program::types::{
        scalar_to_payout_numerators, MakerFill, OrderSide, OutcomeMetadata, ScalarResolutionParams,
    };

    fn test_program_id() -> Pubkey {
        LightconeEnv::default().program_id()
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
            manager: Pubkey::new_unique(),
            num_outcomes: 3,
            oracle: Pubkey::new_unique(),
            question_id: [42u8; 32],
        };
        let program_id = test_program_id();

        let ix = build_create_market_ix(&params, 0, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 5);
        assert_eq!(ix.data.len(), 66); // 1 + 1 + 32 + 32
        assert_eq!(ix.data[0], instruction::CREATE_MARKET);
        assert_eq!(ix.data[1], 3);
    }

    #[test]
    fn test_build_create_market_invalid_outcomes() {
        let params = CreateMarketParams {
            manager: Pubkey::new_unique(),
            num_outcomes: 7, // Invalid - max is 6
            oracle: Pubkey::new_unique(),
            question_id: [0u8; 32],
        };
        let program_id = test_program_id();

        let result = build_create_market_ix(&params, 0, &program_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_add_deposit_mint_ix() {
        let program_id = test_program_id();
        let market = Pubkey::new_unique();
        let params = AddDepositMintParams {
            manager: Pubkey::new_unique(),
            deposit_mint: Pubkey::new_unique(),
            outcome_metadata: vec![
                OutcomeMetadata {
                    name: "Yes".to_string(),
                    symbol: "YES".to_string(),
                    uri: String::new(),
                },
                OutcomeMetadata {
                    name: "No".to_string(),
                    symbol: "NO".to_string(),
                    uri: String::new(),
                },
            ],
        };

        let ix = build_add_deposit_mint_ix(&params, &market, 2, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 12);
        assert!(!ix.accounts[2].is_writable);
        assert_eq!(ix.data[0], instruction::ADD_DEPOSIT_MINT);
    }

    #[test]
    fn test_build_activate_market_ix() {
        let params = ActivateMarketParams {
            manager: Pubkey::new_unique(),
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
            payout_numerators: vec![7, 3],
        };
        let program_id = test_program_id();

        let ix = build_settle_market_ix(&params, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 3);
        assert!(ix.accounts[0].is_signer);
        assert!(!ix.accounts[0].is_writable);
        assert_eq!(ix.data.len(), 9);
        assert_eq!(ix.data[0], instruction::SETTLE_MARKET);
        assert_eq!(&ix.data[1..5], &7u32.to_le_bytes());
        assert_eq!(&ix.data[5..9], &3u32.to_le_bytes());
    }

    #[test]
    fn test_build_settle_market_rejects_invalid_vectors() {
        let program_id = test_program_id();
        let oracle = Pubkey::new_unique();

        for payout_numerators in [vec![], vec![0, 0], vec![1], vec![1; 7]] {
            let params = SettleMarketParams {
                oracle,
                market_id: 1,
                payout_numerators,
            };
            assert!(build_settle_market_ix(&params, &program_id).is_err());
        }
    }

    #[test]
    fn test_winner_takes_all_payout_numerators() {
        let params = SettleMarketParams::winner_takes_all(Pubkey::new_unique(), 1, 2, 4).unwrap();
        assert_eq!(params.payout_numerators, vec![0, 0, 1, 0]);
    }

    #[test]
    fn test_scalar_to_payout_numerators() {
        let params = ScalarResolutionParams {
            min_value: 0,
            max_value: 100,
            resolved_value: 25,
            lower_outcome_index: 0,
            upper_outcome_index: 1,
            num_outcomes: 2,
        };
        assert_eq!(scalar_to_payout_numerators(params).unwrap(), vec![3, 1]);

        let clamped_low = ScalarResolutionParams {
            resolved_value: -5,
            ..params
        };
        assert_eq!(
            scalar_to_payout_numerators(clamped_low).unwrap(),
            vec![1, 0]
        );

        let clamped_high = ScalarResolutionParams {
            resolved_value: 120,
            ..params
        };
        assert_eq!(
            scalar_to_payout_numerators(clamped_high).unwrap(),
            vec![0, 1]
        );
    }

    #[test]
    fn test_signed_scalar_to_payout_numerators_reduces() {
        let params = ScalarResolutionParams {
            min_value: -10_000,
            max_value: 40_000,
            resolved_value: 15_250,
            lower_outcome_index: 0,
            upper_outcome_index: 1,
            num_outcomes: 2,
        };

        assert_eq!(scalar_to_payout_numerators(params).unwrap(), vec![99, 101]);
    }

    #[test]
    fn test_build_redeem_winnings_ix_includes_outcome_and_exchange() {
        let program_id = test_program_id();
        let params = RedeemWinningsParams {
            user: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            deposit_mint: Pubkey::new_unique(),
            amount: 1_000,
        };
        let (exchange, _) = get_exchange_pda(&program_id);

        let ix = build_redeem_winnings_ix(&params, 1, &program_id);

        assert_eq!(ix.accounts.len(), 12);
        assert_eq!(ix.accounts[11].pubkey, exchange);
        assert!(!ix.accounts[5].is_writable);
        assert_eq!(ix.data.len(), 10);
        assert_eq!(ix.data[0], instruction::REDEEM_WINNINGS);
        assert_eq!(&ix.data[1..9], &1_000u64.to_le_bytes());
        assert_eq!(ix.data[9], 1);
    }

    #[test]
    fn test_build_cancel_order_ix() {
        let maker = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let program_id = test_program_id();

        let order = OrderPayload {
            nonce: 1,
            salt: 0,
            maker,
            market,
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: [0u8; 64],
        };

        let operator = Pubkey::new_unique();
        let ix = build_cancel_order_ix(&operator, &market, &order, &program_id);

        assert_eq!(ix.accounts.len(), 4);
        assert_eq!(ix.data.len(), 266); // 1 + 32 + 233
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
            manager: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            mint_a: Pubkey::new_from_array([2u8; 32]),
            mint_b: Pubkey::new_from_array([1u8; 32]),
            mint_a_deposit_mint: Pubkey::new_from_array([12u8; 32]),
            mint_b_deposit_mint: Pubkey::new_from_array([11u8; 32]),
            recent_slot: 12345,
            base_index: 0,
            mint_a_outcome_index: 2,
            mint_b_outcome_index: 1,
        };

        let ix = build_create_orderbook_ix(&params, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 11);
        assert_eq!(ix.data.len(), 12); // 1 + 8 + 1 + 1 + 1
        assert_eq!(ix.data[0], instruction::CREATE_ORDERBOOK);
        assert_eq!(ix.accounts[2].pubkey, params.mint_b);
        assert_eq!(ix.accounts[3].pubkey, params.mint_a);
        assert_eq!(ix.data[9], 1); // base_index after canonical sorting
        assert_eq!(ix.data[10], 1); // canonical mint_a outcome index
        assert_eq!(ix.data[11], 2); // canonical mint_b outcome index
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
    fn test_build_set_manager_ix() {
        let program_id = test_program_id();
        let params = SetManagerParams {
            authority: Pubkey::new_unique(),
            new_manager: Pubkey::new_unique(),
        };

        let ix = build_set_manager_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 2);
        assert_eq!(ix.data.len(), 33);
        assert_eq!(ix.data[0], instruction::SET_MANAGER);
        assert_eq!(&ix.data[1..33], params.new_manager.as_ref());
    }

    #[test]
    fn test_build_match_orders_multi_ix_data_format() {
        let program_id = test_program_id();
        let operator = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let taker = OrderPayload {
            nonce: 1,
            salt: 0,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: OrderSide::Bid,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: [1u8; 64],
        };

        let maker = OrderPayload {
            nonce: 2,
            salt: 0,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: OrderSide::Ask,
            amount_in: 50,
            amount_out: 100,
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

        // Data: 1 + 37 + 64 + 1 + 1 + 117 = 221
        assert_eq!(ix.data.len(), 221);
        assert_eq!(ix.data[0], instruction::MATCH_ORDERS_MULTI);

        // With bitmask=0 (no full fills):
        // Taker: 13 accounts, Maker: 5 accounts = 18 total
        assert_eq!(ix.accounts.len(), 18);
    }

    #[test]
    fn test_build_match_orders_multi_ix_full_fill() {
        let program_id = test_program_id();
        let operator = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let taker = OrderPayload {
            nonce: 1,
            salt: 0,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: OrderSide::Bid,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: [1u8; 64],
        };

        let maker = OrderPayload {
            nonce: 2,
            salt: 0,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: OrderSide::Ask,
            amount_in: 50,
            amount_out: 100,
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
        // Taker: 12 accounts (no order_status), Maker: 4 accounts (no order_status) = 16 total
        assert_eq!(ix.accounts.len(), 16);
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
        assert!(ix.accounts[1].is_writable);
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

        assert_eq!(ix.accounts.len(), 8);
        assert_eq!(ix.data.len(), 9);
        assert_eq!(ix.data[0], instruction::DEPOSIT_TO_GLOBAL);
    }

    #[test]
    fn test_build_deposit_to_global_ix_with_alt_create() {
        let program_id = test_program_id();
        let recent_slot = 12345;
        let params = DepositToGlobalParams {
            user: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            amount: 1_000_000,
        };
        let (user_nonce, _) = get_user_nonce_pda(&params.user, &program_id);
        let (lookup_table, _) = get_alt_pda(&user_nonce, recent_slot);

        let ix = build_deposit_to_global_ix_with_alt(
            &params,
            DepositToGlobalAltContext::Create { recent_slot },
            &program_id,
        );

        assert_eq!(ix.accounts.len(), 11);
        assert_eq!(ix.accounts[8].pubkey, user_nonce);
        assert_eq!(ix.accounts[9].pubkey, lookup_table);
        assert_eq!(ix.data.len(), 17);
        assert_eq!(ix.data[0], instruction::DEPOSIT_TO_GLOBAL);
        assert_eq!(&ix.data[9..17], &recent_slot.to_le_bytes());
    }

    #[test]
    fn test_build_withdraw_from_global_ix() {
        let program_id = test_program_id();
        let params = WithdrawFromGlobalParams {
            user: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            amount: 1_000_000,
        };

        let ix = build_withdraw_from_global_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 7);
        assert_eq!(ix.data.len(), 9);
        assert_eq!(ix.data[0], instruction::WITHDRAW_FROM_GLOBAL);
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
        let deposit_mint = Pubkey::new_unique();
        let params = InitPositionTokensParams {
            payer: Pubkey::new_unique(),
            user: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            deposit_mints: vec![deposit_mint],
            recent_slot: 12345,
        };

        let ix = build_init_position_tokens_ix(&params, 3, &program_id);

        // 11 fixed + 1*(3 + 3*2) = 11 + 9 = 20
        assert_eq!(ix.accounts.len(), 20);
        assert_eq!(ix.data.len(), 10); // 1 + 8 + 1
        assert_eq!(ix.data[0], instruction::INIT_POSITION_TOKENS);
        assert_eq!(ix.data[9], 1); // num_deposit_mints
    }

    #[test]
    fn test_build_deposit_and_swap_ix() {
        let program_id = test_program_id();
        let market = Pubkey::new_unique();
        let deposit_mint = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let taker = OrderPayload {
            nonce: 1,
            salt: 0,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: OrderSide::Bid,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: [1u8; 64],
        };

        let maker_order = OrderPayload {
            nonce: 2,
            salt: 0,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side: OrderSide::Ask,
            amount_in: 50,
            amount_out: 100,
            expiration: 0,
            signature: [2u8; 64],
        };

        let params = DepositAndSwapParams {
            operator: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            taker_order: taker,
            taker_is_full_fill: false,
            taker_is_deposit: true,
            taker_deposit_mint: deposit_mint,
            num_outcomes: 3,
            makers: vec![MakerFill {
                order: maker_order,
                maker_fill_amount: 50,
                taker_fill_amount: 100,
                is_full_fill: false,
                is_deposit: true,
                deposit_mint,
            }],
        };

        let ix = build_deposit_and_swap_ix(&params, &program_id).unwrap();

        // Data: 1 + 37 + 64 + 1 + 1 + 1 + 117 = 222
        assert_eq!(ix.data.len(), 222);
        assert_eq!(ix.data[0], instruction::DEPOSIT_AND_SWAP);

        // Account layout (taker+maker both depositing, no full fills):
        // Fixed: 6
        // Taker order_status: 1
        // Taker common: 8 (nonce, position, base_mint, quote_mint, receive_ata, give_ata, token_2022, system)
        // Taker deposit: 4 + 3*2 = 10 (dm, vault, gdt, global_deposit, cond_mint+ata*3)
        // Maker order_status: 1
        // Maker common: 2 (nonce, position)
        // Maker deposit: 4 + 3*2 = 10
        // Maker swap: 2 (receive_ata, give_ata)
        // Total: 6 + 1 + 8 + 10 + 1 + 2 + 10 + 2 = 40
        assert_eq!(ix.accounts.len(), 40);
    }
}
