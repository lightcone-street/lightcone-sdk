//! Instruction builders for all Lightcone Pinocchio instructions.
//!
//! This module provides functions to build transaction instructions for interacting
//! with the Lightcone Pinocchio program.
//!
//! # Event transport trailer
//!
//! Every public instruction ends with two read-only, non-signer accounts that the
//! program requires for its authenticated event transport: the event-authority
//! PDA (seed `__event_authority`) followed by the executable program account.
//! The program pops both before dispatch, signs one final event-batch self-CPI
//! with the PDA, and rejects a missing, wrong, or writable trailer before any
//! state change (on-chain errors 46 and 68). Public instructions require
//! transaction-level invocation except for the governance allowlist documented
//! in this module's README. Unsupported CPI calls fail with on-chain error 73. Every builder here appends the trailer through the
//! private `public_instruction` constructor, so it always occupies the last
//! two account slots.

use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

// System program ID
fn system_program_id() -> Pubkey {
    solana_system_interface::program::ID
}

use crate::program::constants::{
    instruction, ASSOCIATED_TOKEN_PROGRAM_ID, MAX_MAKERS, MAX_OUTCOMES, MIN_OUTCOMES,
    MPL_TOKEN_METADATA_PROGRAM_ID, PARTICIPANT_MASK_LEN, RENT_SYSVAR_ID, TAKER_MASK,
    TOKEN_PROGRAM_ID,
};
use crate::program::error::{SdkError, SdkResult};
use crate::program::orders::OrderPayload;
use crate::program::pda::{
    get_condition_tombstone_pda, get_conditional_mint_pda, get_event_authority_pda,
    get_exchange_pda, get_global_deposit_token_pda, get_market_pda, get_mint_authority_pda,
    get_mpl_metadata_pda, get_order_status_pda, get_orderbook_pda, get_position_pda,
    get_user_global_deposit_pda, get_vault_pda,
};
use crate::program::types::{
    AcceptRoleParams, ActivateMarketParams, AddDepositMintParams, BuildDepositParams,
    BuildMergeParams, CloseOrderStatusParams, CloseOrderbookParams,
    ClosePositionTokenAccountsParams, ConditionalMetadataParams, CreateMarketParams,
    CreateOrderbookParams, DepositAndSwapParams, DepositToGlobalParams,
    GlobalToMarketDepositParams, InitPositionTokensParams, MatchOrdersMultiParams,
    RedeemWinningsParams, SetAuthorityParams, SetDepositTokenStatusParams, SetFeeReceiverParams,
    SetFeeReceiverWithAtasParams, SetManagerParams, SetMarketFeesParams, SetOracleParams,
    SettleMarketParams, WhitelistDepositTokenParams, WithdrawConditionalFromPositionParams,
    WithdrawFromGlobalParams, WithdrawFromPositionParams,
};
use crate::program::utils::{
    get_conditional_token_ata, get_deposit_token_ata, serialize_conditional_metadata,
    validate_fee_pair, validate_oracle, validate_outcome_count,
};
use crate::program::{derive_condition_id, ORDER_SIZE, SIGNATURE_SIZE, SIGNED_ORDER_SIZE};

// ============================================================================
// Helper Functions
// ============================================================================

/// MatchOrdersMulti body header after the discriminator: taker compact order,
/// taker signature, maker count, and full-fill mask (100 bytes).
const MATCH_ORDER_HEADER_SIZE: usize = ORDER_SIZE + SIGNATURE_SIZE + 1 + PARTICIPANT_MASK_LEN;
/// DepositAndSwap body header: the match header plus the deposit mask (102 bytes).
const DEPOSIT_AND_SWAP_HEADER_SIZE: usize = MATCH_ORDER_HEADER_SIZE + PARTICIPANT_MASK_LEN;
/// Per-maker record: compact order, signature, maker fill, taker fill (113 bytes).
const MAKER_MATCH_SIZE: usize = ORDER_SIZE + SIGNATURE_SIZE + 16;
/// CancelOrder data: discriminator, order hash, signed order (258 bytes).
const CANCEL_ORDER_DATA_SIZE: usize = 1 + 32 + SIGNED_ORDER_SIZE;

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

/// Build a public Lightcone instruction, appending the event transport trailer.
///
/// The program pops the last two accounts of every public instruction before
/// dispatch: the event-authority PDA (`["__event_authority"]`, read-only, never
/// a signer) and the executable program account itself (read-only). It signs
/// its final event-batch self-CPI with that PDA, so an instruction without the
/// trailer fails closed before any state change. Routing every builder through
/// this constructor keeps that invariant in one place.
fn public_instruction(
    program_id: &Pubkey,
    mut accounts: Vec<AccountMeta>,
    data: Vec<u8>,
) -> Instruction {
    let (event_authority, _) = get_event_authority_pda(program_id);
    accounts.reserve_exact(2);
    accounts.push(readonly(event_authority));
    accounts.push(readonly(*program_id));
    Instruction {
        program_id: *program_id,
        accounts,
        data,
    }
}

fn zero_pubkey() -> Pubkey {
    Pubkey::new_from_array([0u8; 32])
}

struct OrderbookMintInput {
    mint: Pubkey,
    deposit_mint: Pubkey,
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

        if params.mint_a_deposit_mint == params.mint_b_deposit_mint {
            return Err(SdkError::DepositMintMismatch);
        }
        if params.outcome_index >= MAX_OUTCOMES {
            return Err(SdkError::InvalidOutcomeIndex {
                index: params.outcome_index,
                max: MAX_OUTCOMES - 1,
            });
        }

        let left = OrderbookMintInput {
            mint: params.mint_a,
            deposit_mint: params.mint_a_deposit_mint,
            is_base: params.base_index == 0,
        };
        let right = OrderbookMintInput {
            mint: params.mint_b,
            deposit_mint: params.mint_b_deposit_mint,
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

/// Derive fixed trading GDTs in canonical conditional-mint order.
fn trading_gdts(
    base_mint: &Pubkey,
    quote_mint: &Pubkey,
    base_deposit_mint: &Pubkey,
    quote_deposit_mint: &Pubkey,
    program_id: &Pubkey,
) -> SdkResult<(Pubkey, Pubkey)> {
    if base_mint == quote_mint {
        return Err(SdkError::InvalidOrderbook);
    }
    if base_deposit_mint == quote_deposit_mint {
        return Err(SdkError::DepositMintMismatch);
    }
    let (deposit_a, deposit_b) = if base_mint.as_ref() < quote_mint.as_ref() {
        (base_deposit_mint, quote_deposit_mint)
    } else {
        (quote_deposit_mint, base_deposit_mint)
    };
    Ok((
        get_global_deposit_token_pda(deposit_a, program_id).0,
        get_global_deposit_token_pda(deposit_b, program_id).0,
    ))
}

fn validate_participant_mask(mask: u16, maker_count: usize) -> SdkResult<()> {
    let allowed = ((1u16 << maker_count) - 1) | TAKER_MASK;
    if mask & !allowed != 0 {
        return Err(SdkError::Serialization(format!(
            "invalid participant mask {mask:#06x} for {maker_count} makers"
        )));
    }
    Ok(())
}

fn validate_trade_order(
    order: &OrderPayload,
    market: &Pubkey,
    base_mint: &Pubkey,
    quote_mint: &Pubkey,
) -> SdkResult<()> {
    if order.market != *market || order.base_mint != *base_mint || order.quote_mint != *quote_mint {
        return Err(SdkError::InvalidOrderbook);
    }
    Ok(())
}

fn validate_funding_mint(
    order: &OrderPayload,
    deposit_mint: &Pubkey,
    base_deposit_mint: &Pubkey,
    quote_deposit_mint: &Pubkey,
) -> SdkResult<()> {
    let expected = match order.side {
        crate::program::types::OrderSide::Bid => quote_deposit_mint,
        crate::program::types::OrderSide::Ask => base_deposit_mint,
    };
    if deposit_mint != expected {
        return Err(SdkError::DepositMintMismatch);
    }
    Ok(())
}

// ============================================================================
// Instruction Builders
// ============================================================================

/// Build Initialize instruction.
///
/// Creates the exchange account (singleton). The authority must match the
/// on-chain `INITIALIZE_AUTHORITY` constant.
///
/// Accounts:
/// 0. authority (signer, mut) - Initial admin
/// 1. exchange (mut) - Exchange PDA
/// 2. system_program (readonly)
/// 3. event_authority (readonly) - Event transport trailer
/// 4. program (readonly) - Event transport trailer
pub fn build_initialize_ix(authority: &Pubkey, program_id: &Pubkey) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![
        signer_mut(*authority),
        writable(exchange),
        readonly(system_program_id()),
    ];

    let data = vec![instruction::INITIALIZE];

    public_instruction(program_id, keys, data)
}

/// Build CreateMarket instruction.
///
/// Creates a new market in Pending status. Rejects zero or off-curve oracles.
///
/// Accounts:
/// 0. manager (signer, mut) - Must be exchange manager
/// 1. exchange (mut) - Exchange PDA
/// 2. market (mut) - Market PDA
/// 3. system_program (readonly)
/// 4. condition_tombstone (mut) - Condition uniqueness PDA
/// 5. event_authority (readonly) - Event transport trailer
/// 6. program (readonly) - Event transport trailer
pub fn build_create_market_ix(
    params: &CreateMarketParams,
    market_id: u64,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    validate_outcome_count(params.num_outcomes)?;
    validate_oracle(&params.oracle)?;
    validate_fee_pair(params.maker_fee_bps, params.taker_fee_bps)?;

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

    // Data: [discriminator, num_outcomes (u8), oracle (32), question_id (32), maker_fee_bps (i16), taker_fee_bps (i16)]
    let mut data = Vec::with_capacity(70);
    data.push(instruction::CREATE_MARKET);
    data.push(params.num_outcomes);
    data.extend_from_slice(params.oracle.as_ref());
    data.extend_from_slice(&params.question_id);
    data.extend_from_slice(&params.maker_fee_bps.to_le_bytes());
    data.extend_from_slice(&params.taker_fee_bps.to_le_bytes());

    Ok(public_instruction(program_id, keys, data))
}

/// Build AddDepositMint instruction.
///
/// Sets up vault and conditional mints for a deposit token.
/// Manager-only — must be called by the exchange manager.
///
/// Accounts:
/// 0. manager (signer, mut) - Must be exchange manager
/// 1. exchange (readonly) - Exchange PDA
/// 2. market (mut) - deposit_mint_count is incremented
/// 3. deposit_mint (readonly)
/// 4. vault (mut)
/// 5. mint_authority (readonly)
/// 6. token_program (SPL Token)
/// 7. system_program
/// 8. global_deposit_token
/// 9+ conditional_mints\[0..num_outcomes\]
/// + event_authority (readonly), program (readonly) - Event transport trailer
pub fn build_add_deposit_mint_ix(
    params: &AddDepositMintParams,
    market: &Pubkey,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    validate_outcome_count(num_outcomes)?;

    let (exchange, _) = get_exchange_pda(program_id);
    let (vault, _) = get_vault_pda(&params.deposit_mint, market, program_id);
    let (mint_authority, _) = get_mint_authority_pda(market, program_id);
    let (global_deposit_token, _) = get_global_deposit_token_pda(&params.deposit_mint, program_id);

    let mut keys = vec![
        signer_mut(params.manager),
        readonly(exchange),
        writable(*market),
        readonly(params.deposit_mint),
        writable(vault),
        readonly(mint_authority),
        readonly(TOKEN_PROGRAM_ID),
        readonly(system_program_id()),
        readonly(global_deposit_token),
    ];

    // Add conditional mints
    for i in 0..num_outcomes {
        let (mint, _) = get_conditional_mint_pda(market, &params.deposit_mint, i, program_id);
        keys.push(writable(mint));
    }

    let data = vec![instruction::ADD_DEPOSIT_MINT];

    Ok(public_instruction(program_id, keys, data))
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
/// 7. mint_authority
/// 8. token_program
/// 9. associated_token_program
/// 10. system_program
/// + remaining accounts (conditional_mint, position_conditional_ata) pairs
/// + event_authority (readonly), program (readonly) - Event transport trailer
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

    let mut keys = vec![
        signer_mut(params.user),
        readonly(exchange),
        readonly(params.market),
        readonly(params.deposit_mint),
        writable(vault),
        writable(user_deposit_ata),
        writable(position),
        readonly(mint_authority),
        readonly(TOKEN_PROGRAM_ID),
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

    public_instruction(program_id, keys, data)
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

    public_instruction(program_id, keys, data)
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
/// 4. event_authority (readonly) - Event transport trailer
/// 5. program (readonly) - Event transport trailer
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

    // Data: [discriminator(1), order_hash(32), OrderPayload(225)] = 258 bytes.
    // The program recomputes the hash from the signed order and rejects a mismatch.
    let mut data = Vec::with_capacity(CANCEL_ORDER_DATA_SIZE);
    data.push(instruction::CANCEL_ORDER);
    data.extend_from_slice(&order_hash);
    data.extend_from_slice(&order.serialize());

    public_instruction(program_id, keys, data)
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

    Ok(public_instruction(program_id, keys, data))
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
        readonly(exchange),
    ];

    let mut data = Vec::with_capacity(10);
    data.push(instruction::REDEEM_WINNINGS);
    data.extend_from_slice(&params.amount.to_le_bytes());
    data.push(outcome_index);

    public_instruction(program_id, keys, data)
}

/// Build SetPaused instruction.
///
/// Admin: pause/unpause exchange.
pub fn build_set_paused_ix(authority: &Pubkey, paused: bool, program_id: &Pubkey) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![signer_mut(*authority), writable(exchange)];

    let data = vec![instruction::SET_PAUSED, if paused { 1 } else { 0 }];

    public_instruction(program_id, keys, data)
}

/// Build SetOperator instruction.
///
/// Admin: propose a new operator. The active operator changes only after the
/// proposed operator signs `AcceptOperator`.
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

    public_instruction(program_id, keys, data)
}

/// Build WithdrawConditionalFromPosition instruction.
///
/// Withdraw conditional tokens from a position ATA to the user's canonical ATA.
/// The conditional mint is derived from `(market, deposit_mint, outcome_index)`.
///
/// Accounts (11):
/// 0. user (signer, writable)
/// 1. exchange (readonly)
/// 2. market (readonly)
/// 3. position (readonly)
/// 4. deposit_mint (readonly)
/// 5. conditional_mint (readonly)
/// 6. position_conditional_ata (writable)
/// 7. user_conditional_ata (writable)
/// 8. token_program (readonly)
/// 9. event_authority (readonly) - Event transport trailer
/// 10. program (readonly) - Event transport trailer
pub fn build_withdraw_conditional_from_position_ix(
    params: &WithdrawConditionalFromPositionParams,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);
    let (conditional_mint, _) = get_conditional_mint_pda(
        &params.market,
        &params.deposit_mint,
        params.outcome_index,
        program_id,
    );
    let position_conditional_ata = get_conditional_token_ata(&position, &conditional_mint);
    let user_conditional_ata = get_conditional_token_ata(&params.user, &conditional_mint);

    let keys = vec![
        signer_mut(params.user),
        readonly(exchange),
        readonly(params.market),
        readonly(position),
        readonly(params.deposit_mint),
        readonly(conditional_mint),
        writable(position_conditional_ata),
        writable(user_conditional_ata),
        readonly(TOKEN_PROGRAM_ID),
    ];

    // Data: [discriminator(1), amount(8), outcome_index(1)] = 10 bytes
    let mut data = Vec::with_capacity(10);
    data.push(instruction::WITHDRAW_CONDITIONAL_FROM_POSITION);
    data.extend_from_slice(&params.amount.to_le_bytes());
    data.push(params.outcome_index);

    public_instruction(program_id, keys, data)
}

/// Build WithdrawConditionalFromPosition instruction.
///
/// Compatibility wrapper for the previous SDK function name.
pub fn build_withdraw_from_position_ix(
    params: &WithdrawFromPositionParams,
    program_id: &Pubkey,
) -> Instruction {
    build_withdraw_conditional_from_position_ix(params, program_id)
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

    public_instruction(program_id, keys, data)
}

/// Build MatchOrdersMulti instruction.
///
/// Match taker against makers.
///
/// Data format (101 + 113 * M bytes for M makers):
/// ```text
/// [0]       discriminator
/// [1..34]   taker Order (33 bytes)
/// [34..98]  taker_signature (64 bytes)
/// [98]      num_makers
/// [99..101] full_fill_bitmask (u16, little-endian)
/// Per maker (113 bytes each):
///   [+0..+33]    maker Order (33)
///   [+33..+97]   maker_signature (64)
///   [+97..+105]  maker_fill_amount (8)
///   [+105..+113] taker_fill_amount (8)
/// ```
///
/// Accounts (17 + 4 * M - F, where F counts full-fill participants, which
/// omit their order status):
/// ```text
///   Fixed: operator, exchange, market, orderbook, GDT A, GDT B
///   Taker: [order_status], position, base_mint, quote_mint, base_ata, quote_ata,
///          token_program, system_program, fee_receiver_quote_ata, fee_receiver,
///          ata_program
///   Per maker: [order_status], position, base_ata, quote_ata
/// ```
/// The event transport trailer (event_authority, program) is always appended last.
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

    validate_trade_order(
        &params.taker_order,
        &params.market,
        &params.base_mint,
        &params.quote_mint,
    )?;
    for maker in &params.maker_orders {
        validate_trade_order(maker, &params.market, &params.base_mint, &params.quote_mint)?;
        if maker.side == params.taker_order.side {
            return Err(SdkError::InvalidSide(maker.side as u8));
        }
    }
    validate_participant_mask(params.full_fill_bitmask, params.maker_orders.len())?;
    let (gdt_a, gdt_b) = trading_gdts(
        &params.base_mint,
        &params.quote_mint,
        &params.base_deposit_mint,
        &params.quote_deposit_mint,
        program_id,
    )?;

    let (exchange, _) = get_exchange_pda(program_id);
    let (orderbook, _) = get_orderbook_pda(&params.base_mint, &params.quote_mint, program_id);
    let taker_order_hash = params.taker_order.hash();
    let (taker_position, _) =
        get_position_pda(&params.taker_order.maker, &params.market, program_id);
    let taker_base_ata = get_conditional_token_ata(&taker_position, &params.base_mint);
    let taker_quote_ata = get_conditional_token_ata(&taker_position, &params.quote_mint);
    let fee_receiver_quote_ata =
        get_conditional_token_ata(&params.fee_receiver, &params.quote_mint);

    let taker_full_fill = params.full_fill_bitmask & TAKER_MASK != 0;

    let mut keys = Vec::new();

    // Taker fixed accounts
    keys.push(signer_mut(params.operator));
    keys.push(readonly(exchange));
    keys.push(readonly(params.market));
    keys.push(readonly(orderbook));
    keys.push(readonly(gdt_a));
    keys.push(readonly(gdt_b));

    if !taker_full_fill {
        // A clear taker bit includes its writable status account.
        let (taker_order_status, _) = get_order_status_pda(&taker_order_hash, program_id);
        keys.push(writable(taker_order_status));
    }
    // Remaining taker accounts
    keys.push(readonly(taker_position));
    keys.push(readonly(params.base_mint));
    keys.push(readonly(params.quote_mint));
    keys.push(writable(taker_base_ata));
    keys.push(writable(taker_quote_ata));
    keys.push(readonly(TOKEN_PROGRAM_ID));
    keys.push(readonly(system_program_id()));
    keys.push(writable(fee_receiver_quote_ata));
    keys.push(readonly(params.fee_receiver));
    keys.push(readonly(ASSOCIATED_TOKEN_PROGRAM_ID));

    // Per-maker accounts
    for (i, maker_order) in params.maker_orders.iter().enumerate() {
        let maker_full_fill = (params.full_fill_bitmask >> i) & 1 == 1;

        if !maker_full_fill {
            // bit i = 0: 4 accounts (order_status, position, base_ata, quote_ata)
            let maker_order_hash = maker_order.hash();
            let (maker_order_status, _) = get_order_status_pda(&maker_order_hash, program_id);
            keys.push(writable(maker_order_status));
        }
        // bit i = 1: 3 accounts (position, base_ata, quote_ata)
        let (maker_position, _) = get_position_pda(&maker_order.maker, &params.market, program_id);
        let maker_base_ata = get_conditional_token_ata(&maker_position, &params.base_mint);
        let maker_quote_ata = get_conditional_token_ata(&maker_position, &params.quote_mint);

        keys.push(readonly(maker_position));
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
    data.extend_from_slice(&params.full_fill_bitmask.to_le_bytes());

    for (i, maker_order) in params.maker_orders.iter().enumerate() {
        let maker_compact = maker_order.to_order();

        data.extend_from_slice(&maker_compact.serialize());
        data.extend_from_slice(&maker_order.signature);
        data.extend_from_slice(&params.maker_fill_amounts[i].to_le_bytes());
        data.extend_from_slice(&params.taker_fill_amounts[i].to_le_bytes());
    }

    Ok(public_instruction(program_id, keys, data))
}

/// Build an orderbook for one market outcome and two distinct collateral assets.
///
/// Canonicalize the supplied conditional pair while preserving collateral and
/// base orientation. The manager pays for the book and fee receiver quote ATA.
/// Registered collateral may be inactive at creation. Trading validates activity.
///
/// Business accounts: manager, market, mint A, mint B, book, GDT A, GDT B,
/// exchange, system program, collateral A, collateral B, token program,
/// ATA program, fee receiver, fee receiver quote ATA. The event trailers follow.
pub fn build_create_orderbook_ix(
    params: &CreateOrderbookParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    let canonical = CanonicalOrderbookMints::from_params(params)?;
    for (mint, deposit_mint) in [
        (&params.mint_a, &params.mint_a_deposit_mint),
        (&params.mint_b, &params.mint_b_deposit_mint),
    ] {
        if get_conditional_mint_pda(
            &params.market,
            deposit_mint,
            params.outcome_index,
            program_id,
        )
        .0 != *mint
        {
            return Err(SdkError::InvalidConditionalMint);
        }
    }
    let (exchange, _) = get_exchange_pda(program_id);
    let (orderbook, _) =
        get_orderbook_pda(&canonical.mint_a.mint, &canonical.mint_b.mint, program_id);
    let (gdt_a, _) = get_global_deposit_token_pda(&canonical.mint_a.deposit_mint, program_id);
    let (gdt_b, _) = get_global_deposit_token_pda(&canonical.mint_b.deposit_mint, program_id);
    let quote_mint = if canonical.base_index() == 0 {
        canonical.mint_b.mint
    } else {
        canonical.mint_a.mint
    };
    let fee_receiver_quote_ata = get_conditional_token_ata(&params.fee_receiver, &quote_mint);

    let keys = vec![
        signer_mut(params.manager),
        readonly(params.market),
        readonly(canonical.mint_a.mint),
        readonly(canonical.mint_b.mint),
        writable(orderbook),
        readonly(gdt_a),
        readonly(gdt_b),
        readonly(exchange),
        readonly(system_program_id()),
        readonly(canonical.mint_a.deposit_mint),
        readonly(canonical.mint_b.deposit_mint),
        readonly(TOKEN_PROGRAM_ID),
        readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
        readonly(params.fee_receiver),
        writable(fee_receiver_quote_ata),
    ];

    let data = vec![
        instruction::CREATE_ORDERBOOK,
        canonical.base_index(),
        params.outcome_index,
    ];

    Ok(public_instruction(program_id, keys, data))
}

/// Build SetAuthority instruction.
///
/// Propose a new exchange authority. The active authority changes only after
/// the proposed authority signs `AcceptAuthority`.
///
/// Accounts (4):
/// 0. authority (signer)
/// 1. exchange (mut)
/// 2. event_authority (readonly) - Event transport trailer
/// 3. program (readonly) - Event transport trailer
pub fn build_set_authority_ix(params: &SetAuthorityParams, program_id: &Pubkey) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![signer_mut(params.current_authority), writable(exchange)];

    // Data: [discriminator(1), new_authority(32)] = 33 bytes
    let mut data = Vec::with_capacity(33);
    data.push(instruction::SET_AUTHORITY);
    data.extend_from_slice(params.new_authority.as_ref());

    public_instruction(program_id, keys, data)
}

/// Build SetManager instruction.
///
/// Propose a new exchange manager. The active manager changes only after the
/// proposed manager signs `AcceptManager`.
///
/// Accounts (4):
/// 0. authority (signer)
/// 1. exchange (mut)
/// 2. event_authority (readonly) - Event transport trailer
/// 3. program (readonly) - Event transport trailer
pub fn build_set_manager_ix(params: &SetManagerParams, program_id: &Pubkey) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![signer_mut(params.authority), writable(exchange)];

    let mut data = Vec::with_capacity(33);
    data.push(instruction::SET_MANAGER);
    data.extend_from_slice(params.new_manager.as_ref());

    public_instruction(program_id, keys, data)
}

fn build_accept_role_ix(
    params: &AcceptRoleParams,
    discriminator: u8,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![signer(params.incoming_role), writable(exchange)];

    public_instruction(program_id, keys, vec![discriminator])
}

/// Build AcceptAuthority instruction.
pub fn build_accept_authority_ix(params: &AcceptRoleParams, program_id: &Pubkey) -> Instruction {
    build_accept_role_ix(params, instruction::ACCEPT_AUTHORITY, program_id)
}

/// Build AcceptManager instruction.
pub fn build_accept_manager_ix(params: &AcceptRoleParams, program_id: &Pubkey) -> Instruction {
    build_accept_role_ix(params, instruction::ACCEPT_MANAGER, program_id)
}

/// Build AcceptOperator instruction.
pub fn build_accept_operator_ix(params: &AcceptRoleParams, program_id: &Pubkey) -> Instruction {
    build_accept_role_ix(params, instruction::ACCEPT_OPERATOR, program_id)
}

/// Build SetOracle instruction.
///
/// Authority-only. Reassigns a market oracle while the market is not resolved
/// or cancelled. Rejects zero or off-curve oracles. The condition ID stays unchanged.
pub fn build_set_oracle_ix(
    params: &SetOracleParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    validate_oracle(&params.new_oracle)?;

    let (exchange, _) = get_exchange_pda(program_id);
    let keys = vec![
        signer(params.authority),
        readonly(exchange),
        writable(params.market),
    ];

    let mut data = Vec::with_capacity(33);
    data.push(instruction::SET_ORACLE);
    data.extend_from_slice(params.new_oracle.as_ref());

    Ok(public_instruction(program_id, keys, data))
}

/// Build SetMarketFees instruction.
///
/// Manager-only. Updates one or more markets in one instruction.
pub fn build_set_market_fees_ix(
    params: &SetMarketFeesParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.updates.is_empty() {
        return Err(SdkError::MissingField("updates".to_string()));
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let mut keys = Vec::with_capacity(4 + params.updates.len());
    keys.push(signer_mut(params.manager));
    keys.push(readonly(exchange));

    let mut data = Vec::with_capacity(1 + params.updates.len() * 4);
    data.push(instruction::SET_MARKET_FEES);
    for update in &params.updates {
        validate_fee_pair(update.maker_fee_bps, update.taker_fee_bps)?;
        keys.push(writable(update.market));
        data.extend_from_slice(&update.maker_fee_bps.to_le_bytes());
        data.extend_from_slice(&update.taker_fee_bps.to_le_bytes());
    }

    Ok(public_instruction(program_id, keys, data))
}

/// Build SetFeeReceiver instruction.
///
/// Authority-only. New orderbooks and match instructions must use this receiver's quote ATA.
pub fn build_set_fee_receiver_ix(
    params: &SetFeeReceiverParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.new_fee_receiver == zero_pubkey() {
        return Err(SdkError::InvalidFeeReceiver);
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let keys = vec![signer_mut(params.authority), writable(exchange)];

    let mut data = Vec::with_capacity(33);
    data.push(instruction::SET_FEE_RECEIVER);
    data.extend_from_slice(params.new_fee_receiver.as_ref());

    Ok(public_instruction(program_id, keys, data))
}

/// Build SetFeeReceiver instruction with optional ATA creation accounts.
///
/// This non-breaking variant preserves the same instruction discriminator and
/// data as `build_set_fee_receiver_ix`, while appending the optional account
/// block used by the on-chain program to create receiver quote ATAs.
pub fn build_set_fee_receiver_with_atas_ix(
    params: &SetFeeReceiverWithAtasParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.new_fee_receiver == zero_pubkey() {
        return Err(SdkError::InvalidFeeReceiver);
    }
    if params.quote_mints.is_empty() {
        return Err(SdkError::MissingField("quote_mints".to_string()));
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let mut keys = Vec::with_capacity(8 + params.quote_mints.len() * 2);
    keys.push(signer_mut(params.authority));
    keys.push(writable(exchange));
    keys.push(readonly(params.new_fee_receiver));
    keys.push(readonly(TOKEN_PROGRAM_ID));
    keys.push(readonly(ASSOCIATED_TOKEN_PROGRAM_ID));
    keys.push(readonly(system_program_id()));

    for quote_mint in &params.quote_mints {
        let fee_receiver_quote_ata =
            get_conditional_token_ata(&params.new_fee_receiver, quote_mint);
        keys.push(readonly(*quote_mint));
        keys.push(writable(fee_receiver_quote_ata));
    }

    let mut data = Vec::with_capacity(33);
    data.push(instruction::SET_FEE_RECEIVER);
    data.extend_from_slice(params.new_fee_receiver.as_ref());

    Ok(public_instruction(program_id, keys, data))
}

/// Build CreateConditionalMetadata instruction.
pub fn build_create_conditional_metadata_ix(
    params: &ConditionalMetadataParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    build_conditional_metadata_ix(params, true, program_id)
}

/// Build UpdateConditionalMetadata instruction.
pub fn build_update_conditional_metadata_ix(
    params: &ConditionalMetadataParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    build_conditional_metadata_ix(params, false, program_id)
}

fn build_conditional_metadata_ix(
    params: &ConditionalMetadataParams,
    is_create: bool,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.outcome_index >= MAX_OUTCOMES {
        return Err(SdkError::InvalidOutcomeIndex {
            index: params.outcome_index,
            max: MAX_OUTCOMES - 1,
        });
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let (conditional_mint, _) = get_conditional_mint_pda(
        &params.market,
        &params.deposit_mint,
        params.outcome_index,
        program_id,
    );
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);
    let (metadata, _) = get_mpl_metadata_pda(&conditional_mint);

    let mut data =
        Vec::with_capacity(2 + 12 + params.name.len() + params.symbol.len() + params.uri.len());
    data.push(if is_create {
        instruction::CREATE_CONDITIONAL_METADATA
    } else {
        instruction::UPDATE_CONDITIONAL_METADATA
    });
    data.push(params.outcome_index);
    data.extend(serialize_conditional_metadata(
        &params.name,
        &params.symbol,
        &params.uri,
    )?);

    let mut keys = vec![
        if is_create {
            signer_mut(params.manager)
        } else {
            signer(params.manager)
        },
        readonly(exchange),
        readonly(params.market),
        readonly(params.deposit_mint),
        readonly(conditional_mint),
        writable(metadata),
        readonly(mint_authority),
        readonly(*MPL_TOKEN_METADATA_PROGRAM_ID),
    ];

    if is_create {
        keys.push(readonly(system_program_id()));
        keys.push(readonly(RENT_SYSVAR_ID));
    }

    Ok(public_instruction(program_id, keys, data))
}

/// Build WhitelistDepositToken instruction.
///
/// Admin: whitelist a token mint for global deposits.
///
/// Accounts (7):
/// 0. authority (signer, mut) - Must be exchange authority
/// 1. exchange (mut) - Exchange PDA; increments deposit_token_count
/// 2. mint (readonly) - Token mint to whitelist
/// 3. global_deposit_token (mut) - PDA to create ["global_deposit", mint]
/// 4. system_program (readonly)
/// 5. event_authority (readonly) - Event transport trailer
/// 6. program (readonly) - Event transport trailer
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

    public_instruction(program_id, keys, data)
}

/// Build SetDepositTokenStatus instruction.
///
/// Manager-only. Enable or disable trading backed by the registered collateral.
/// Deposits, preparation, splits, merges, withdrawals, and redemption retain
/// their existing rules and do not require an active collateral flag.
pub fn build_set_deposit_token_status_ix(
    params: &SetDepositTokenStatusParams,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (global_deposit_token, _) = get_global_deposit_token_pda(&params.mint, program_id);

    let keys = vec![
        signer(params.manager),
        readonly(exchange),
        writable(global_deposit_token),
    ];

    let data = vec![
        instruction::SET_DEPOSIT_TOKEN_STATUS,
        if params.active { 1 } else { 0 },
    ];

    public_instruction(program_id, keys, data)
}

/// Build DepositToGlobal instruction.
///
/// Deposit tokens from user's token account into their global deposit PDA.
///
/// Accounts (10 including the event trailers):
/// 0. user (signer, mut)
/// 1. global_deposit_token (readonly) - Whitelist PDA
/// 2. mint (readonly)
/// 3. user_global_deposit (mut) - User's deposit PDA
/// 4. user_token_account (mut) - User's source token account
/// 5. token_program (readonly)
/// 6. system_program (readonly)
/// 7. exchange (readonly) - Exchange PDA for pause validation
/// + event_authority (readonly), program (readonly) - Event transport trailer (always last)
pub fn build_deposit_to_global_ix(
    params: &DepositToGlobalParams,
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
        readonly(system_program_id()),
        readonly(exchange),
    ];

    let mut data = Vec::with_capacity(9);
    data.push(instruction::DEPOSIT_TO_GLOBAL);
    data.extend_from_slice(&params.amount.to_le_bytes());

    public_instruction(program_id, keys, data)
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
/// 8. mint_authority (readonly)
/// 9. token_program (readonly)
/// 10. ata_program (readonly)
/// 11. system_program (readonly)
/// + per outcome: conditional_mint[i] (mut), position_conditional_ata[i] (mut)
/// + event_authority (readonly), program (readonly) - Event transport trailer
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
        readonly(mint_authority),
        readonly(TOKEN_PROGRAM_ID),
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

    public_instruction(program_id, keys, data)
}

/// Build idempotent position and conditional-ATA preparation.
///
/// The signing payer may sponsor an on-curve user who does not sign. Each call
/// validates every requested collateral group and creates missing accounts.
/// Supply groups in strictly increasing GDT index order and outcomes from zero.
///
/// This infallible raw API leaves input validation to the program. Use
/// `InitPositionTokensBuilder::build_ix` or `Positions::init_position_tokens_tx`
/// for local beneficiary, group-count, and outcome-count validation.
///
/// Business prefix: payer, user, exchange, market, position, mint authority,
/// token program, ATA program, system program. Each group adds collateral,
/// vault, GDT, and conditional mint/position ATA pairs for every outcome.
/// The event trailers always follow all groups.
pub fn build_init_position_tokens_ix(
    params: &InitPositionTokensParams,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);

    let mut keys = vec![
        signer_mut(params.payer),
        readonly(params.user),
        readonly(exchange),
        readonly(params.market),
        writable(position),
        readonly(mint_authority),
        readonly(TOKEN_PROGRAM_ID),
        readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
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

    let mut data = Vec::with_capacity(2);
    data.push(instruction::INIT_POSITION_TOKENS);
    data.push(params.deposit_mints.len() as u8);

    public_instruction(program_id, keys, data)
}

/// Build DepositAndSwap instruction.
///
/// Unified order execution: participants can deposit from global deposits and/or swap
/// conditional tokens in a single instruction. Each participant's deposit is conditional
/// on the deposit_bitmask.
///
/// Data format (103 + 113 * M bytes): the MatchOrdersMulti body with the u16
/// deposit mask after the full-fill mask, so maker records start at byte 103.
///
/// Account layout (18 + 4 * M - F + D * (4 + 2 * O) before the trailer, where
/// F counts full-fill participants, D depositors, and O market outcomes):
///   Fixed (11): operator, exchange, market, orderbook, GDT A, GDT B, mint_authority, token_program,
///              fee_receiver_quote_ata, fee_receiver, ata_program
///   Taker block: [order_status], position, base_mint, quote_mint,
///                taker_receive_ata, taker_give_ata, system_program
///   Taker deposit block (optional): deposit_mint, vault, gdt, user_global_deposit,
///                                    [cond_mint, ata] × num_outcomes
///   Per-maker blocks: [order_status], position,
///                      [deposit block if depositing],
///                      maker_receive_ata, maker_give_ata
///   Trailer (2): event_authority, program (always last)
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

    validate_outcome_count(params.num_outcomes)?;
    validate_trade_order(
        &params.taker_order,
        &params.market,
        &params.base_mint,
        &params.quote_mint,
    )?;
    for maker in &params.makers {
        validate_trade_order(
            &maker.order,
            &params.market,
            &params.base_mint,
            &params.quote_mint,
        )?;
        if maker.order.side == params.taker_order.side {
            return Err(SdkError::InvalidSide(maker.order.side as u8));
        }
    }
    let (gdt_a, gdt_b) = trading_gdts(
        &params.base_mint,
        &params.quote_mint,
        &params.base_deposit_mint,
        &params.quote_deposit_mint,
        program_id,
    )?;
    if params.taker_is_deposit {
        validate_funding_mint(
            &params.taker_order,
            &params.taker_deposit_mint,
            &params.base_deposit_mint,
            &params.quote_deposit_mint,
        )?;
    }
    for maker in &params.makers {
        if maker.is_deposit {
            validate_funding_mint(
                &maker.order,
                &maker.deposit_mint,
                &params.base_deposit_mint,
                &params.quote_deposit_mint,
            )?;
        }
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let (orderbook, _) = get_orderbook_pda(&params.base_mint, &params.quote_mint, program_id);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);
    let (taker_position, _) =
        get_position_pda(&params.taker_order.maker, &params.market, program_id);
    let fee_receiver_quote_ata =
        get_conditional_token_ata(&params.fee_receiver, &params.quote_mint);

    let taker_side = params.taker_order.side as u8;
    let (receive_mint, give_mint) = if taker_side == 0 {
        (&params.base_mint, &params.quote_mint)
    } else {
        (&params.quote_mint, &params.base_mint)
    };

    // Build bitmasks
    let mut full_fill_bitmask: u16 = 0;
    let mut deposit_bitmask: u16 = 0;
    if params.taker_is_full_fill {
        full_fill_bitmask |= TAKER_MASK;
    }
    if params.taker_is_deposit {
        deposit_bitmask |= TAKER_MASK;
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

    // Fixed accounts (11)
    keys.push(signer_mut(params.operator));
    keys.push(readonly(exchange));
    keys.push(readonly(params.market));
    keys.push(readonly(orderbook));
    keys.push(readonly(gdt_a));
    keys.push(readonly(gdt_b));
    keys.push(readonly(mint_authority));
    keys.push(readonly(TOKEN_PROGRAM_ID));
    keys.push(writable(fee_receiver_quote_ata));
    keys.push(readonly(params.fee_receiver));
    keys.push(readonly(ASSOCIATED_TOKEN_PROGRAM_ID));

    // Taker order_status (only if not full fill)
    if !params.taker_is_full_fill {
        let taker_order_hash = params.taker_order.hash();
        let (taker_order_status, _) = get_order_status_pda(&taker_order_hash, program_id);
        keys.push(writable(taker_order_status));
    }

    // Taker common block
    let taker_receive_ata = get_conditional_token_ata(&taker_position, receive_mint);
    let taker_give_ata = get_conditional_token_ata(&taker_position, give_mint);
    keys.push(readonly(taker_position));
    keys.push(readonly(params.base_mint));
    keys.push(readonly(params.quote_mint));
    keys.push(writable(taker_receive_ata));
    keys.push(writable(taker_give_ata));
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
        let (maker_position, _) = get_position_pda(&maker.order.maker, &params.market, program_id);

        if !maker.is_full_fill {
            let maker_order_hash = maker.order.hash();
            let (maker_order_status, _) = get_order_status_pda(&maker_order_hash, program_id);
            keys.push(writable(maker_order_status));
        }

        keys.push(readonly(maker_position));

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
    data.extend_from_slice(&full_fill_bitmask.to_le_bytes());
    data.extend_from_slice(&deposit_bitmask.to_le_bytes());

    for maker in &params.makers {
        let maker_compact = maker.order.to_order();
        data.extend_from_slice(&maker_compact.serialize());
        data.extend_from_slice(&maker.order.signature);
        data.extend_from_slice(&maker.maker_fill_amount.to_le_bytes());
        data.extend_from_slice(&maker.taker_fill_amount.to_le_bytes());
    }

    Ok(public_instruction(program_id, keys, data))
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

    public_instruction(program_id, keys, data)
}

/// Build CloseOrderStatus instruction.
///
/// Closes a fully-filled, non-cancelled order status PDA and returns rent to
/// the operator.
pub fn build_close_order_status_ix(
    params: &CloseOrderStatusParams,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let (order_status, _) = get_order_status_pda(&params.order_hash, program_id);

    let keys = vec![
        signer_mut(params.operator),
        readonly(exchange),
        writable(order_status),
    ];

    let mut data = Vec::with_capacity(33);
    data.push(instruction::CLOSE_ORDER_STATUS);
    data.extend_from_slice(&params.order_hash);

    public_instruction(program_id, keys, data)
}

/// Build ClosePositionTokenAccounts instruction.
///
/// Attempts to close empty SPL conditional ATAs owned by a position PDA
/// after market resolution. Non-empty token accounts are skipped by the program.
pub fn build_close_position_token_accounts_ix(
    params: &ClosePositionTokenAccountsParams,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    validate_outcome_count(num_outcomes)?;
    if params.deposit_mints.is_empty() {
        return Err(SdkError::MissingField("deposit_mints".to_string()));
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let mut keys = vec![
        signer_mut(params.operator),
        readonly(exchange),
        readonly(params.market),
        readonly(params.position),
        readonly(TOKEN_PROGRAM_ID),
    ];

    for deposit_mint in &params.deposit_mints {
        keys.push(readonly(*deposit_mint));
        for i in 0..num_outcomes {
            let (conditional_mint, _) =
                get_conditional_mint_pda(&params.market, deposit_mint, i, program_id);
            keys.push(readonly(conditional_mint));
            let position_ata = get_conditional_token_ata(&params.position, &conditional_mint);
            keys.push(writable(position_ata));
        }
    }

    Ok(public_instruction(
        program_id,
        keys,
        vec![instruction::CLOSE_POSITION_TOKEN_ACCOUNTS],
    ))
}

/// Build CloseOrderbook instruction.
///
/// Close the orderbook PDA after its market resolves and refund its operator.
pub fn build_close_orderbook_ix(params: &CloseOrderbookParams, program_id: &Pubkey) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![
        signer_mut(params.operator),
        readonly(exchange),
        writable(params.orderbook),
        readonly(params.market),
    ];

    public_instruction(program_id, keys, vec![instruction::CLOSE_ORDERBOOK])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::LightconeEnv;
    use crate::program::types::{
        scalar_to_payout_numerators, MakerFill, MarketFeeUpdate, OrderSide, ScalarResolutionParams,
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
        assert_eq!(ix.accounts.len(), 5);
        assert_eq!(ix.data, vec![instruction::INITIALIZE]);
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
            oracle: *crate::program::constants::INITIALIZE_AUTHORITY,
            question_id: [42u8; 32],
            maker_fee_bps: 10,
            taker_fee_bps: 20,
        };
        let program_id = test_program_id();

        let ix = build_create_market_ix(&params, 0, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 7);
        assert_eq!(ix.data.len(), 70); // 1 + 1 + 32 + 32 + 2 + 2
        assert_eq!(ix.data[0], instruction::CREATE_MARKET);
        assert_eq!(ix.data[1], 3);
        assert_eq!(&ix.data[66..68], &10i16.to_le_bytes());
        assert_eq!(&ix.data[68..70], &20i16.to_le_bytes());
    }

    #[test]
    fn test_build_create_market_invalid_outcomes() {
        let params = CreateMarketParams {
            manager: Pubkey::new_unique(),
            num_outcomes: 7, // Invalid - max is 6
            oracle: *crate::program::constants::INITIALIZE_AUTHORITY,
            question_id: [0u8; 32],
            maker_fee_bps: 0,
            taker_fee_bps: 0,
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
        };

        let ix = build_add_deposit_mint_ix(&params, &market, 2, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 13);
        assert_eq!(ix.accounts[2].pubkey, market);
        assert!(ix.accounts[2].is_writable);
        assert_eq!(ix.data, vec![instruction::ADD_DEPOSIT_MINT]);
    }

    #[test]
    fn test_build_activate_market_ix() {
        let params = ActivateMarketParams {
            manager: Pubkey::new_unique(),
            market_id: 5,
        };
        let program_id = test_program_id();

        let ix = build_activate_market_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 5);
        assert_eq!(ix.data, vec![instruction::ACTIVATE_MARKET]);
    }

    #[test]
    fn test_build_settle_market_ix() {
        let params = SettleMarketParams {
            oracle: *crate::program::constants::INITIALIZE_AUTHORITY,
            market_id: 1,
            payout_numerators: vec![7, 3],
        };
        let program_id = test_program_id();

        let ix = build_settle_market_ix(&params, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 5);
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
        let oracle = *crate::program::constants::INITIALIZE_AUTHORITY;

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
            user: *crate::program::constants::INITIALIZE_AUTHORITY,
            market: Pubkey::new_unique(),
            deposit_mint: Pubkey::new_unique(),
            amount: 1_000,
        };
        let (exchange, _) = get_exchange_pda(&program_id);

        let ix = build_redeem_winnings_ix(&params, 1, &program_id);

        assert_eq!(ix.accounts.len(), 13);
        assert_eq!(ix.accounts[10].pubkey, exchange);
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
            salt: 1,
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

        assert_eq!(ix.accounts.len(), 6);
        assert_eq!(ix.data.len(), 258); // 1 + 32 + 225
        assert_eq!(ix.data[0], instruction::CANCEL_ORDER);
        assert_eq!(&ix.data[1..33], &order.hash());
        assert_eq!(&ix.data[33..], &order.serialize());
    }

    #[test]
    fn test_build_withdraw_from_position_ix() {
        let program_id = test_program_id();
        let user = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let deposit_mint = Pubkey::new_unique();
        let outcome_index = 0;
        let params = WithdrawConditionalFromPositionParams {
            user,
            market,
            deposit_mint,
            amount: 1000,
            outcome_index,
        };

        let ix = build_withdraw_conditional_from_position_ix(&params, &program_id);

        let (exchange, _) = get_exchange_pda(&program_id);
        let (position, _) = get_position_pda(&user, &market, &program_id);
        let (conditional_mint, _) =
            get_conditional_mint_pda(&market, &deposit_mint, outcome_index, &program_id);
        let position_conditional_ata = get_conditional_token_ata(&position, &conditional_mint);
        let user_conditional_ata = get_conditional_token_ata(&user, &conditional_mint);

        assert_eq!(ix.accounts.len(), 11);
        assert_eq!(ix.accounts[0], signer_mut(user));
        assert_eq!(ix.accounts[1], readonly(exchange));
        assert_eq!(ix.accounts[2], readonly(market));
        assert_eq!(ix.accounts[3], readonly(position));
        assert_eq!(ix.accounts[4], readonly(deposit_mint));
        assert_eq!(ix.accounts[5], readonly(conditional_mint));
        assert_eq!(ix.accounts[6], writable(position_conditional_ata));
        assert_eq!(ix.accounts[7], writable(user_conditional_ata));
        assert_eq!(ix.accounts[8], readonly(TOKEN_PROGRAM_ID));
        assert_eq!(ix.data.len(), 10); // 1 + 8 + 1
        assert_eq!(ix.data[0], instruction::WITHDRAW_CONDITIONAL_FROM_POSITION);
        assert_eq!(&ix.data[1..9], &1000u64.to_le_bytes());
        assert_eq!(ix.data[9], outcome_index);
    }

    #[test]
    fn test_build_set_authority_ix() {
        let program_id = test_program_id();
        let params = SetAuthorityParams {
            current_authority: Pubkey::new_unique(),
            new_authority: Pubkey::new_unique(),
        };

        let ix = build_set_authority_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 4);
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

        assert_eq!(ix.accounts.len(), 4);
        assert_eq!(ix.data.len(), 33);
        assert_eq!(ix.data[0], instruction::SET_MANAGER);
        assert_eq!(&ix.data[1..33], params.new_manager.as_ref());
    }

    #[test]
    fn test_build_accept_role_ixs() {
        let program_id = test_program_id();
        let incoming_role = Pubkey::new_unique();
        let params = AcceptRoleParams { incoming_role };

        let authority_ix = build_accept_authority_ix(&params, &program_id);
        let manager_ix = build_accept_manager_ix(&params, &program_id);
        let operator_ix = build_accept_operator_ix(&params, &program_id);

        for ix in [&authority_ix, &manager_ix, &operator_ix] {
            assert_eq!(ix.accounts.len(), 4);
            assert_eq!(ix.accounts[0].pubkey, incoming_role);
            assert!(ix.accounts[0].is_signer);
            assert!(!ix.accounts[0].is_writable);
            assert!(ix.accounts[1].is_writable);
            assert_eq!(ix.data.len(), 1);
        }
        assert_eq!(authority_ix.data[0], instruction::ACCEPT_AUTHORITY);
        assert_eq!(manager_ix.data[0], instruction::ACCEPT_MANAGER);
        assert_eq!(operator_ix.data[0], instruction::ACCEPT_OPERATOR);
    }

    #[test]
    fn test_build_set_oracle_ix() {
        let program_id = test_program_id();
        let params = SetOracleParams {
            authority: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            new_oracle: *crate::program::constants::INITIALIZE_AUTHORITY,
        };

        let ix = build_set_oracle_ix(&params, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 5);
        assert_eq!(ix.accounts[0].pubkey, params.authority);
        assert!(ix.accounts[0].is_signer);
        assert!(!ix.accounts[0].is_writable);
        assert!(ix.accounts[2].is_writable);
        assert_eq!(ix.data.len(), 33);
        assert_eq!(ix.data[0], instruction::SET_ORACLE);
        assert_eq!(&ix.data[1..33], params.new_oracle.as_ref());
    }

    #[test]
    fn test_build_set_oracle_ix_rejects_zero_oracle() {
        let program_id = test_program_id();
        let params = SetOracleParams {
            authority: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            new_oracle: zero_pubkey(),
        };

        assert!(matches!(
            build_set_oracle_ix(&params, &program_id),
            Err(SdkError::InvalidOracle)
        ));
    }

    #[test]
    fn test_build_set_market_fees_ix() {
        let program_id = test_program_id();
        let market = Pubkey::new_unique();
        let params = SetMarketFeesParams {
            manager: Pubkey::new_unique(),
            updates: vec![MarketFeeUpdate {
                market,
                maker_fee_bps: -10,
                taker_fee_bps: 25,
            }],
        };

        let ix = build_set_market_fees_ix(&params, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 5);
        assert_eq!(ix.accounts[2].pubkey, market);
        assert_eq!(ix.data[0], instruction::SET_MARKET_FEES);
        assert_eq!(&ix.data[1..3], &(-10i16).to_le_bytes());
        assert_eq!(&ix.data[3..5], &25i16.to_le_bytes());
    }

    #[test]
    fn test_build_set_fee_receiver_ix() {
        let program_id = test_program_id();
        let params = SetFeeReceiverParams {
            authority: Pubkey::new_unique(),
            new_fee_receiver: Pubkey::new_unique(),
        };

        let ix = build_set_fee_receiver_ix(&params, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 4);
        assert_eq!(ix.data.len(), 33);
        assert_eq!(ix.data[0], instruction::SET_FEE_RECEIVER);
        assert_eq!(&ix.data[1..33], params.new_fee_receiver.as_ref());
    }

    #[test]
    fn test_build_set_fee_receiver_with_atas_ix() {
        let program_id = test_program_id();
        let quote_mint_a = Pubkey::new_unique();
        let quote_mint_b = Pubkey::new_unique();
        let params = SetFeeReceiverWithAtasParams {
            authority: Pubkey::new_unique(),
            new_fee_receiver: Pubkey::new_unique(),
            quote_mints: vec![quote_mint_a, quote_mint_b],
        };

        let ix = build_set_fee_receiver_with_atas_ix(&params, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 12);
        assert_eq!(ix.accounts[0].pubkey, params.authority);
        assert!(ix.accounts[0].is_signer);
        assert!(ix.accounts[0].is_writable);
        assert_eq!(ix.accounts[2].pubkey, params.new_fee_receiver);
        assert_eq!(ix.accounts[6].pubkey, quote_mint_a);
        assert_eq!(
            ix.accounts[7].pubkey,
            get_conditional_token_ata(&params.new_fee_receiver, &quote_mint_a)
        );
        assert!(ix.accounts[7].is_writable);
        assert_eq!(ix.accounts[8].pubkey, quote_mint_b);
        assert_eq!(
            ix.accounts[9].pubkey,
            get_conditional_token_ata(&params.new_fee_receiver, &quote_mint_b)
        );
        assert_eq!(ix.data.len(), 33);
        assert_eq!(ix.data[0], instruction::SET_FEE_RECEIVER);
    }

    #[test]
    fn test_build_set_fee_receiver_with_atas_ix_requires_quote_mints() {
        let program_id = test_program_id();
        let params = SetFeeReceiverWithAtasParams {
            authority: Pubkey::new_unique(),
            new_fee_receiver: Pubkey::new_unique(),
            quote_mints: vec![],
        };

        assert!(matches!(
            build_set_fee_receiver_with_atas_ix(&params, &program_id),
            Err(SdkError::MissingField(field)) if field == "quote_mints"
        ));
    }

    #[test]
    fn test_build_conditional_metadata_ixs() {
        let program_id = test_program_id();
        let params = ConditionalMetadataParams {
            manager: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            deposit_mint: Pubkey::new_unique(),
            outcome_index: 1,
            name: "Yes".to_string(),
            symbol: "YES".to_string(),
            uri: "https://example.com/yes.json".to_string(),
        };

        let create_ix = build_create_conditional_metadata_ix(&params, &program_id).unwrap();
        assert_eq!(create_ix.accounts.len(), 12);
        assert_eq!(create_ix.data[0], instruction::CREATE_CONDITIONAL_METADATA);
        assert_eq!(create_ix.data[1], 1);
        assert_eq!(
            u32::from_le_bytes(create_ix.data[2..6].try_into().unwrap()),
            3
        );

        let update_ix = build_update_conditional_metadata_ix(&params, &program_id).unwrap();
        assert_eq!(update_ix.accounts.len(), 10);
        assert_eq!(update_ix.data[0], instruction::UPDATE_CONDITIONAL_METADATA);
        assert!(!update_ix.accounts[0].is_writable);
    }

    #[test]
    fn test_build_match_orders_multi_ix_data_format() {
        let program_id = test_program_id();
        let operator = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let taker = OrderPayload {
            salt: 1,
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
            salt: 2,
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
            base_deposit_mint: base_mint,
            quote_deposit_mint: quote_mint,
            operator,
            market,
            base_mint,
            quote_mint,
            fee_receiver: Pubkey::new_unique(),
            taker_order: taker,
            maker_orders: vec![maker],
            maker_fill_amounts: vec![50],
            taker_fill_amounts: vec![100],
            full_fill_bitmask: 0,
        };

        let ix = build_match_orders_multi_ix(&params, &program_id).unwrap();

        // Data: 1 + 33 + 64 + 1 + 2 + 113 = 214
        assert_eq!(ix.data.len(), 214);
        assert_eq!(ix.data[0], instruction::MATCH_ORDERS_MULTI);

        // With bitmask=0 (no full fills):
        // Taker: 17 accounts, Maker: 4 accounts, trailer: 2 accounts = 23 total
        assert_eq!(ix.accounts.len(), 23);
    }

    #[test]
    fn test_build_match_orders_multi_ix_full_fill() {
        let program_id = test_program_id();
        let operator = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let taker = OrderPayload {
            salt: 1,
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
            salt: 2,
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

        // bit 0 = 1 (maker 0 full fill), bit 15 = 1 (taker full fill)
        let params = MatchOrdersMultiParams {
            base_deposit_mint: base_mint,
            quote_deposit_mint: quote_mint,
            operator,
            market,
            base_mint,
            quote_mint,
            fee_receiver: Pubkey::new_unique(),
            taker_order: taker,
            maker_orders: vec![maker],
            maker_fill_amounts: vec![50],
            taker_fill_amounts: vec![100],
            full_fill_bitmask: 0x8001,
        };

        let ix = build_match_orders_multi_ix(&params, &program_id).unwrap();

        // With bitmask=0x8001 (taker + maker 0 full fill):
        // Taker: 16 accounts (no order_status), Maker: 3 accounts (no order_status),
        // trailer: 2 accounts = 21 total
        assert_eq!(ix.accounts.len(), 21);
    }

    #[test]
    fn test_match_orders_multi_eleven_makers_and_u16_mask() {
        let program_id = test_program_id();
        let mut params = eleven_maker_params();
        let ix = build_match_orders_multi_ix(&params, &program_id).unwrap();

        assert_eq!(ix.data[0], instruction::MATCH_ORDERS_MULTI);
        assert_eq!(ix.data.len(), 1344);
        assert_eq!(&ix.data[99..101], &[0x81, 0x85]);
        assert_eq!(
            u16::from_le_bytes(ix.data[99..101].try_into().unwrap()),
            0x8581
        );
        assert_trade_records(
            &ix.data,
            101,
            &params.taker_order,
            &params.maker_orders,
            &params.maker_fill_amounts,
            &params.taker_fill_amounts,
        );
        // 17 + 4*11 - 5 full fills, followed by the two-account trailer.
        assert_eq!(ix.accounts.len(), 58);
        assert_eleven_maker_account_sequence(&ix, &params, &program_id, false);

        for invalid_mask in [0x0800, 0x1000, 0x2000, 0x4000] {
            params.full_fill_bitmask = invalid_mask;
            assert!(matches!(
                build_match_orders_multi_ix(&params, &program_id),
                Err(SdkError::Serialization(_))
            ));
        }

        params.full_fill_bitmask = 0x8581;
        params.maker_orders.push(params.maker_orders[0].clone());
        params.maker_fill_amounts.push(1);
        params.taker_fill_amounts.push(2);
        assert!(matches!(
            build_match_orders_multi_ix(&params, &program_id),
            Err(SdkError::TooManyMakers { count: 12 })
        ));
    }

    #[test]
    fn test_build_whitelist_deposit_token_ix() {
        let program_id = test_program_id();
        let params = WhitelistDepositTokenParams {
            authority: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
        };

        let ix = build_whitelist_deposit_token_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 7);
        assert!(ix.accounts[1].is_writable);
        assert_eq!(ix.data, vec![instruction::WHITELIST_DEPOSIT_TOKEN]);
    }

    #[test]
    fn test_build_set_deposit_token_status_ix() {
        let program_id = test_program_id();
        let params = SetDepositTokenStatusParams {
            manager: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            active: false,
        };

        let ix = build_set_deposit_token_status_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 5);
        assert_eq!(ix.accounts[0].pubkey, params.manager);
        assert!(ix.accounts[0].is_signer);
        assert!(!ix.accounts[0].is_writable);
        assert!(ix.accounts[2].is_writable);
        assert_eq!(ix.data, vec![instruction::SET_DEPOSIT_TOKEN_STATUS, 0]);
    }

    #[test]
    fn test_build_deposit_to_global_ix() {
        let program_id = test_program_id();
        let params = DepositToGlobalParams {
            user: *crate::program::constants::INITIALIZE_AUTHORITY,
            mint: Pubkey::new_unique(),
            amount: 1_000_000,
        };

        let ix = build_deposit_to_global_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 10);
        assert_eq!(ix.data.len(), 9);
        assert_eq!(ix.data[0], instruction::DEPOSIT_TO_GLOBAL);
    }

    #[test]
    fn test_build_withdraw_from_global_ix() {
        let program_id = test_program_id();
        let params = WithdrawFromGlobalParams {
            user: *crate::program::constants::INITIALIZE_AUTHORITY,
            mint: Pubkey::new_unique(),
            amount: 1_000_000,
        };

        let ix = build_withdraw_from_global_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 9);
        assert_eq!(ix.data.len(), 9);
        assert_eq!(ix.data[0], instruction::WITHDRAW_FROM_GLOBAL);
    }

    #[test]
    fn test_build_global_to_market_deposit_ix() {
        let program_id = test_program_id();
        let params = GlobalToMarketDepositParams {
            user: *crate::program::constants::INITIALIZE_AUTHORITY,
            market: Pubkey::new_unique(),
            deposit_mint: Pubkey::new_unique(),
            amount: 500_000,
        };

        let ix = build_global_to_market_deposit_ix(&params, 3, &program_id);

        // 12 fixed + 3*2 conditional + 2 trailer = 20
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
            user: *crate::program::constants::INITIALIZE_AUTHORITY,
            market: Pubkey::new_unique(),
            deposit_mints: vec![deposit_mint],
        };

        let ix = build_init_position_tokens_ix(&params, 3, &program_id);

        // 9 fixed + 1*(3 + 3*2) + 2 trailer = 20
        assert_eq!(ix.accounts.len(), 20);
        assert_eq!(ix.data.len(), 2); // discriminator + group count
        assert_eq!(ix.data[0], instruction::INIT_POSITION_TOKENS);
        assert_eq!(ix.data[1], 1); // num_deposit_mints
    }

    #[test]
    fn test_build_deposit_and_swap_ix() {
        let program_id = test_program_id();
        let market = Pubkey::new_unique();
        let deposit_mint = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let taker = OrderPayload {
            salt: 1,
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
            salt: 2,
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
            base_deposit_mint: base_mint,
            quote_deposit_mint: deposit_mint,
            operator: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            fee_receiver: Pubkey::new_unique(),
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
                deposit_mint: base_mint,
            }],
        };

        let ix = build_deposit_and_swap_ix(&params, &program_id).unwrap();

        // Data: 1 + 33 + 64 + 1 + 2 + 2 + 113 = 216
        assert_eq!(ix.data.len(), 216);
        assert_eq!(ix.data[0], instruction::DEPOSIT_AND_SWAP);

        // Account layout (taker+maker both depositing, no full fills):
        // Fixed: 11
        // Taker order_status: 1
        // Taker common: 6 (position, base_mint, quote_mint, receive_ata, give_ata, system)
        // Taker deposit: 4 + 3*2 = 10 (dm, vault, gdt, global_deposit, cond_mint+ata*3)
        // Maker order_status: 1
        // Maker common: 1 (position)
        // Maker deposit: 4 + 3*2 = 10
        // Maker swap: 2 (receive_ata, give_ata)
        // Trailer: 2 (event_authority, program)
        // Total: 11 + 1 + 6 + 10 + 1 + 1 + 10 + 2 + 2 = 44
        assert_eq!(ix.accounts.len(), 44);
    }

    #[test]
    fn test_deposit_and_swap_eleven_makers_and_u16_masks() {
        let program_id = test_program_id();
        let matching = eleven_maker_params();
        let mut params = DepositAndSwapParams {
            operator: matching.operator,
            market: matching.market,
            base_mint: matching.base_mint,
            quote_mint: matching.quote_mint,
            base_deposit_mint: matching.base_deposit_mint,
            quote_deposit_mint: matching.quote_deposit_mint,
            fee_receiver: matching.fee_receiver,
            taker_order: matching.taker_order.clone(),
            taker_is_full_fill: true,
            taker_is_deposit: true,
            taker_deposit_mint: matching.quote_deposit_mint,
            num_outcomes: 6,
            makers: matching
                .maker_orders
                .iter()
                .enumerate()
                .map(|(i, order)| MakerFill {
                    order: order.clone(),
                    maker_fill_amount: matching.maker_fill_amounts[i],
                    taker_fill_amount: matching.taker_fill_amounts[i],
                    is_full_fill: matches!(i, 0 | 7 | 8 | 10),
                    is_deposit: matches!(i, 1 | 9 | 10),
                    deposit_mint: matching.base_deposit_mint,
                })
                .collect(),
        };
        let ix = build_deposit_and_swap_ix(&params, &program_id).unwrap();

        assert_eq!(ix.data[0], instruction::DEPOSIT_AND_SWAP);
        assert_eq!(ix.data.len(), 1346);
        assert_eq!(&ix.data[99..103], &[0x81, 0x85, 0x02, 0x86]);
        assert_eq!(
            u16::from_le_bytes(ix.data[99..101].try_into().unwrap()),
            0x8581
        );
        assert_eq!(
            u16::from_le_bytes(ix.data[101..103].try_into().unwrap()),
            0x8602
        );
        assert_trade_records(
            &ix.data,
            103,
            &matching.taker_order,
            &matching.maker_orders,
            &matching.maker_fill_amounts,
            &matching.taker_fill_amounts,
        );
        // Four depositors each supply 4 + 2*6 references, including repeated
        // mints and GDTs: 18 + 4*11 - 5 full fills + 4*16 + 2 trailer accounts.
        assert_eq!(ix.accounts.len(), 123);
        assert_eleven_maker_account_sequence(&ix, &matching, &program_id, true);

        params.makers.push(params.makers[0].clone());
        assert!(matches!(
            build_deposit_and_swap_ix(&params, &program_id),
            Err(SdkError::TooManyMakers { count: 12 })
        ));
    }

    #[test]
    fn test_build_close_order_status_ix() {
        let program_id = test_program_id();
        let order_hash = [9u8; 32];
        let params = CloseOrderStatusParams {
            operator: Pubkey::new_unique(),
            order_hash,
        };

        let ix = build_close_order_status_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 5);
        assert_eq!(ix.data.len(), 33);
        assert_eq!(ix.data[0], instruction::CLOSE_ORDER_STATUS);
        assert_eq!(&ix.data[1..33], &order_hash);
    }

    #[test]
    fn test_build_close_position_token_accounts_ix() {
        let program_id = test_program_id();
        let market = Pubkey::new_unique();
        let position = Pubkey::new_unique();
        let deposit_mint = Pubkey::new_unique();
        let params = ClosePositionTokenAccountsParams {
            operator: Pubkey::new_unique(),
            market,
            position,
            deposit_mints: vec![deposit_mint],
        };

        let ix = build_close_position_token_accounts_ix(&params, 3, &program_id).unwrap();

        // 5 fixed + one group of deposit_mint + 3*(conditional_mint, ata) + 2 trailer
        assert_eq!(ix.accounts.len(), 14);
        assert_eq!(ix.data, vec![instruction::CLOSE_POSITION_TOKEN_ACCOUNTS]);
    }

    fn event_transport_trailer(program_id: &Pubkey) -> [AccountMeta; 2] {
        let (event_authority, _) = get_event_authority_pda(program_id);
        [readonly(event_authority), readonly(*program_id)]
    }

    fn sample_order(
        market: Pubkey,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        side: OrderSide,
    ) -> OrderPayload {
        OrderPayload {
            salt: 1,
            maker: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            side,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: [1u8; 64],
        }
    }

    fn eleven_maker_params() -> MatchOrdersMultiParams {
        let market = Pubkey::new_unique();
        // Reverse base/quote relative to canonical mint order so GDT slots 4
        // and 5 must contain quote collateral first, then base collateral.
        let base_mint = Pubkey::new_from_array([2; 32]);
        let quote_mint = Pubkey::new_from_array([1; 32]);
        let mut taker_order = sample_order(market, base_mint, quote_mint, OrderSide::Bid);
        taker_order.salt = 0x0102_0304_0506_0708;
        taker_order.amount_in = u64::MAX - 100;
        taker_order.amount_out = (1u64 << 53) + 101;
        taker_order.expiration = -123_456_789;
        let maker_orders = (0u32..11)
            .map(|i| {
                let mut order = sample_order(market, base_mint, quote_mint, OrderSide::Ask);
                order.salt = 0x1112_1314_1516_1700 + u64::from(i);
                order.amount_in = (1u64 << 53) + 201 + u64::from(i);
                order.amount_out = u64::MAX - 301 - u64::from(i);
                order.expiration = -987_654_321 - i64::from(i);
                order.signature = [i as u8 + 2; 64];
                order
            })
            .collect();
        MatchOrdersMultiParams {
            operator: Pubkey::new_unique(),
            market,
            base_mint,
            quote_mint,
            base_deposit_mint: Pubkey::new_unique(),
            quote_deposit_mint: Pubkey::new_unique(),
            fee_receiver: Pubkey::new_unique(),
            taker_order,
            maker_orders,
            maker_fill_amounts: (0..11).map(|i| (1u64 << 53) + 401 + i).collect(),
            taker_fill_amounts: (0..11).map(|i| u64::MAX - 501 - i).collect(),
            // Full fills for the taker and makers 0, 7, 8, and 10.
            full_fill_bitmask: 0x8581,
        }
    }

    fn assert_encoded_order(data: &[u8], expected: &OrderPayload) {
        assert_eq!(data.len(), 97);
        let order = crate::program::orders::Order::deserialize(&data[..33]).unwrap();
        assert_eq!(order.salt, expected.salt);
        assert_eq!(order.side, expected.side);
        assert_eq!(order.amount_in, expected.amount_in);
        assert_eq!(order.amount_out, expected.amount_out);
        assert_eq!(order.expiration, expected.expiration);
        assert_eq!(&data[33..97], &expected.signature);
    }

    fn assert_trade_records(
        data: &[u8],
        records_offset: usize,
        taker: &OrderPayload,
        makers: &[OrderPayload],
        maker_fills: &[u64],
        taker_fills: &[u64],
    ) {
        assert_eq!(data[98], 11);
        assert_encoded_order(&data[1..98], taker);
        let records = data[records_offset..].chunks_exact(113);
        assert!(records.remainder().is_empty());
        assert_eq!(records.len(), 11);
        for (i, record) in records.enumerate() {
            assert_encoded_order(&record[..97], &makers[i]);
            assert_eq!(
                u64::from_le_bytes(record[97..105].try_into().unwrap()),
                maker_fills[i]
            );
            assert_eq!(
                u64::from_le_bytes(record[105..113].try_into().unwrap()),
                taker_fills[i]
            );
        }
    }

    fn assert_eleven_maker_account_sequence(
        ix: &Instruction,
        params: &MatchOrdersMultiParams,
        program_id: &Pubkey,
        deposit_and_swap: bool,
    ) {
        // Expected (pubkey, signer, writable) triples follow the nonce-free
        // program parsers (refactor/engine-accounting-02 at 9702f23). They do
        // not use SDK account-meta helpers, mask decoding, or the builders'
        // collateral canonicalization.
        let taker = params.taker_order.maker;
        let taker_position = get_position_pda(&taker, &params.market, program_id).0;
        let taker_base_ata = get_conditional_token_ata(&taker_position, &params.base_mint);
        let taker_quote_ata = get_conditional_token_ata(&taker_position, &params.quote_mint);
        let fee_ata = get_conditional_token_ata(&params.fee_receiver, &params.quote_mint);
        let mut expected = vec![
            (params.operator, true, true),
            (get_exchange_pda(program_id).0, false, false),
            (params.market, false, false),
            (
                get_orderbook_pda(&params.base_mint, &params.quote_mint, program_id).0,
                false,
                false,
            ),
            (
                get_global_deposit_token_pda(&params.quote_deposit_mint, program_id).0,
                false,
                false,
            ),
            (
                get_global_deposit_token_pda(&params.base_deposit_mint, program_id).0,
                false,
                false,
            ),
        ];
        if deposit_and_swap {
            expected.extend([
                (
                    get_mint_authority_pda(&params.market, program_id).0,
                    false,
                    false,
                ),
                (TOKEN_PROGRAM_ID, false, false),
                (fee_ata, false, true),
                (params.fee_receiver, false, false),
                (ASSOCIATED_TOKEN_PROGRAM_ID, false, false),
                // The full-fill taker has no status account. Its BUY order
                // receives base and gives quote, defining both parties' ATA order.
                (taker_position, false, false),
                (params.base_mint, false, false),
                (params.quote_mint, false, false),
                (taker_base_ata, false, true),
                (taker_quote_ata, false, true),
                (solana_system_interface::program::ID, false, false),
            ]);
            expected.extend(expected_six_outcome_deposit_accounts(
                &params.market,
                &taker,
                &params.quote_deposit_mint,
                program_id,
            ));
        } else {
            expected.extend([
                (taker_position, false, false),
                (params.base_mint, false, false),
                (params.quote_mint, false, false),
                (taker_base_ata, false, true),
                (taker_quote_ata, false, true),
                (TOKEN_PROGRAM_ID, false, false),
                (solana_system_interface::program::ID, false, false),
                (fee_ata, false, true),
                (params.fee_receiver, false, false),
                (ASSOCIATED_TOKEN_PROGRAM_ID, false, false),
            ]);
        }

        for (i, maker) in params.maker_orders.iter().enumerate() {
            // Literal participants selected by this vector's full-fill mask.
            if !matches!(i, 0 | 7 | 8 | 10) {
                expected.push((
                    get_order_status_pda(&maker.hash(), program_id).0,
                    false,
                    true,
                ));
            }
            let position = get_position_pda(&maker.maker, &params.market, program_id).0;
            expected.push((position, false, false));
            if deposit_and_swap && matches!(i, 1 | 9 | 10) {
                expected.extend(expected_six_outcome_deposit_accounts(
                    &params.market,
                    &maker.maker,
                    &params.base_deposit_mint,
                    program_id,
                ));
            }
            // Settlement ATAs follow every maker block, including depositors.
            expected.extend([
                (
                    get_conditional_token_ata(&position, &params.base_mint),
                    false,
                    true,
                ),
                (
                    get_conditional_token_ata(&position, &params.quote_mint),
                    false,
                    true,
                ),
            ]);
        }
        expected.extend([
            (get_event_authority_pda(program_id).0, false, false),
            (*program_id, false, false),
        ]);
        assert_eq!(ix.accounts.len(), expected.len());
        for (index, (actual, expected)) in ix.accounts.iter().zip(expected).enumerate() {
            assert_eq!(
                (actual.pubkey, actual.is_signer, actual.is_writable),
                expected,
                "account {index} differs from the program ABI"
            );
        }
    }

    fn expected_six_outcome_deposit_accounts(
        market: &Pubkey,
        user: &Pubkey,
        collateral: &Pubkey,
        program_id: &Pubkey,
    ) -> Vec<(Pubkey, bool, bool)> {
        let position = get_position_pda(user, market, program_id).0;
        let mut expected = vec![
            (*collateral, false, false),
            (get_vault_pda(collateral, market, program_id).0, false, true),
            (
                get_global_deposit_token_pda(collateral, program_id).0,
                false,
                false,
            ),
            (
                get_user_global_deposit_pda(user, collateral, program_id).0,
                false,
                true,
            ),
        ];
        for outcome in [0, 1, 2, 3, 4, 5] {
            let mint = get_conditional_mint_pda(market, collateral, outcome, program_id).0;
            expected.extend([
                (mint, false, true),
                (get_conditional_token_ata(&position, &mint), false, true),
            ]);
        }
        expected
    }

    /// One representative instruction per public builder. Register new builders
    /// here so `every_public_builder_ends_with_event_transport_trailer` covers them.
    fn all_public_builders(program_id: &Pubkey) -> Vec<(&'static str, Instruction)> {
        let market = Pubkey::new_unique();
        let deposit_mint = Pubkey::new_unique();
        let base_deposit_mint = Pubkey::new_unique();
        let base_mint = get_conditional_mint_pda(&market, &base_deposit_mint, 0, program_id).0;
        let quote_mint = get_conditional_mint_pda(&market, &deposit_mint, 0, program_id).0;
        let signer = *crate::program::constants::INITIALIZE_AUTHORITY;
        let taker = sample_order(market, base_mint, quote_mint, OrderSide::Bid);
        let maker = sample_order(market, base_mint, quote_mint, OrderSide::Ask);
        let metadata = ConditionalMetadataParams {
            manager: signer,
            market,
            deposit_mint,
            outcome_index: 0,
            name: "Yes".to_string(),
            symbol: "YES".to_string(),
            uri: "https://example.com/yes.json".to_string(),
        };
        let deposit_to_global = DepositToGlobalParams {
            user: signer,
            mint: deposit_mint,
            amount: 1,
        };
        let accept_role = AcceptRoleParams {
            incoming_role: signer,
        };

        vec![
            ("initialize", build_initialize_ix(&signer, program_id)),
            (
                "create_market",
                build_create_market_ix(
                    &CreateMarketParams {
                        manager: signer,
                        num_outcomes: 2,
                        oracle: *crate::program::constants::INITIALIZE_AUTHORITY,
                        question_id: [1u8; 32],
                        maker_fee_bps: 0,
                        taker_fee_bps: 0,
                    },
                    0,
                    program_id,
                )
                .unwrap(),
            ),
            (
                "add_deposit_mint",
                build_add_deposit_mint_ix(
                    &AddDepositMintParams {
                        manager: signer,
                        deposit_mint,
                    },
                    &market,
                    2,
                    program_id,
                )
                .unwrap(),
            ),
            (
                "deposit",
                build_deposit_ix(
                    &BuildDepositParams {
                        user: signer,
                        market,
                        deposit_mint,
                        amount: 1,
                    },
                    2,
                    program_id,
                ),
            ),
            (
                "merge",
                build_merge_ix(
                    &BuildMergeParams {
                        user: signer,
                        market,
                        deposit_mint,
                        amount: 1,
                    },
                    2,
                    program_id,
                ),
            ),
            (
                "cancel_order",
                build_cancel_order_ix(&signer, &market, &taker, program_id),
            ),
            (
                "settle_market",
                build_settle_market_ix(&SettleMarketParams::new(signer, 0, vec![1, 0]), program_id)
                    .unwrap(),
            ),
            (
                "redeem_winnings",
                build_redeem_winnings_ix(
                    &RedeemWinningsParams {
                        user: signer,
                        market,
                        deposit_mint,
                        amount: 1,
                    },
                    0,
                    program_id,
                ),
            ),
            ("set_paused", build_set_paused_ix(&signer, true, program_id)),
            (
                "set_operator",
                build_set_operator_ix(&signer, &Pubkey::new_unique(), program_id),
            ),
            (
                "withdraw_conditional_from_position",
                build_withdraw_conditional_from_position_ix(
                    &WithdrawConditionalFromPositionParams {
                        user: signer,
                        market,
                        deposit_mint,
                        amount: 1,
                        outcome_index: 0,
                    },
                    program_id,
                ),
            ),
            (
                "withdraw_from_position",
                build_withdraw_from_position_ix(
                    &WithdrawFromPositionParams {
                        user: signer,
                        market,
                        deposit_mint,
                        amount: 1,
                        outcome_index: 0,
                    },
                    program_id,
                ),
            ),
            (
                "activate_market",
                build_activate_market_ix(
                    &ActivateMarketParams {
                        manager: signer,
                        market_id: 0,
                    },
                    program_id,
                ),
            ),
            (
                "match_orders_multi",
                build_match_orders_multi_ix(
                    &MatchOrdersMultiParams {
                        base_deposit_mint: base_mint,
                        quote_deposit_mint: quote_mint,
                        operator: signer,
                        market,
                        base_mint,
                        quote_mint,
                        fee_receiver: Pubkey::new_unique(),
                        taker_order: taker.clone(),
                        maker_orders: vec![maker.clone()],
                        maker_fill_amounts: vec![50],
                        taker_fill_amounts: vec![100],
                        full_fill_bitmask: 0,
                    },
                    program_id,
                )
                .unwrap(),
            ),
            (
                "create_orderbook",
                build_create_orderbook_ix(
                    &CreateOrderbookParams {
                        manager: signer,
                        market,
                        mint_a: base_mint,
                        mint_b: quote_mint,
                        fee_receiver: Pubkey::new_unique(),
                        mint_a_deposit_mint: base_deposit_mint,
                        mint_b_deposit_mint: deposit_mint,
                        base_index: 0,
                        outcome_index: 0,
                    },
                    program_id,
                )
                .unwrap(),
            ),
            (
                "set_authority",
                build_set_authority_ix(
                    &SetAuthorityParams {
                        current_authority: signer,
                        new_authority: Pubkey::new_unique(),
                    },
                    program_id,
                ),
            ),
            (
                "set_manager",
                build_set_manager_ix(
                    &SetManagerParams {
                        authority: signer,
                        new_manager: Pubkey::new_unique(),
                    },
                    program_id,
                ),
            ),
            (
                "accept_authority",
                build_accept_authority_ix(&accept_role, program_id),
            ),
            (
                "accept_manager",
                build_accept_manager_ix(&accept_role, program_id),
            ),
            (
                "accept_operator",
                build_accept_operator_ix(&accept_role, program_id),
            ),
            (
                "set_oracle",
                build_set_oracle_ix(
                    &SetOracleParams {
                        authority: signer,
                        market,
                        new_oracle: *crate::program::constants::INITIALIZE_AUTHORITY,
                    },
                    program_id,
                )
                .unwrap(),
            ),
            (
                "set_market_fees",
                build_set_market_fees_ix(
                    &SetMarketFeesParams {
                        manager: signer,
                        updates: vec![MarketFeeUpdate {
                            market,
                            maker_fee_bps: 0,
                            taker_fee_bps: 0,
                        }],
                    },
                    program_id,
                )
                .unwrap(),
            ),
            (
                "set_fee_receiver",
                build_set_fee_receiver_ix(
                    &SetFeeReceiverParams {
                        authority: signer,
                        new_fee_receiver: Pubkey::new_unique(),
                    },
                    program_id,
                )
                .unwrap(),
            ),
            (
                "set_fee_receiver_with_atas",
                build_set_fee_receiver_with_atas_ix(
                    &SetFeeReceiverWithAtasParams {
                        authority: signer,
                        new_fee_receiver: Pubkey::new_unique(),
                        quote_mints: vec![quote_mint],
                    },
                    program_id,
                )
                .unwrap(),
            ),
            (
                "create_conditional_metadata",
                build_create_conditional_metadata_ix(&metadata, program_id).unwrap(),
            ),
            (
                "update_conditional_metadata",
                build_update_conditional_metadata_ix(&metadata, program_id).unwrap(),
            ),
            (
                "whitelist_deposit_token",
                build_whitelist_deposit_token_ix(
                    &WhitelistDepositTokenParams {
                        authority: signer,
                        mint: deposit_mint,
                    },
                    program_id,
                ),
            ),
            (
                "set_deposit_token_status",
                build_set_deposit_token_status_ix(
                    &SetDepositTokenStatusParams {
                        manager: signer,
                        mint: deposit_mint,
                        active: true,
                    },
                    program_id,
                ),
            ),
            (
                "deposit_to_global",
                build_deposit_to_global_ix(&deposit_to_global, program_id),
            ),
            (
                "global_to_market_deposit",
                build_global_to_market_deposit_ix(
                    &GlobalToMarketDepositParams {
                        user: signer,
                        market,
                        deposit_mint,
                        amount: 1,
                    },
                    2,
                    program_id,
                ),
            ),
            (
                "init_position_tokens",
                build_init_position_tokens_ix(
                    &InitPositionTokensParams {
                        payer: signer,
                        user: *crate::program::constants::INITIALIZE_AUTHORITY,
                        market,
                        deposit_mints: vec![deposit_mint],
                    },
                    2,
                    program_id,
                ),
            ),
            (
                "deposit_and_swap",
                build_deposit_and_swap_ix(
                    &DepositAndSwapParams {
                        base_deposit_mint: base_mint,
                        quote_deposit_mint: deposit_mint,
                        operator: signer,
                        market,
                        base_mint,
                        quote_mint,
                        fee_receiver: Pubkey::new_unique(),
                        taker_order: taker,
                        taker_is_full_fill: true,
                        taker_is_deposit: true,
                        taker_deposit_mint: deposit_mint,
                        num_outcomes: 2,
                        makers: vec![MakerFill {
                            order: maker,
                            maker_fill_amount: 50,
                            taker_fill_amount: 100,
                            is_full_fill: true,
                            is_deposit: false,
                            deposit_mint,
                        }],
                    },
                    program_id,
                )
                .unwrap(),
            ),
            (
                "withdraw_from_global",
                build_withdraw_from_global_ix(
                    &WithdrawFromGlobalParams {
                        user: signer,
                        mint: deposit_mint,
                        amount: 1,
                    },
                    program_id,
                ),
            ),
            (
                "close_order_status",
                build_close_order_status_ix(
                    &CloseOrderStatusParams {
                        operator: signer,
                        order_hash: [2u8; 32],
                    },
                    program_id,
                ),
            ),
            (
                "close_position_token_accounts",
                build_close_position_token_accounts_ix(
                    &ClosePositionTokenAccountsParams {
                        operator: signer,
                        market,
                        position: Pubkey::new_unique(),
                        deposit_mints: vec![deposit_mint],
                    },
                    2,
                    program_id,
                )
                .unwrap(),
            ),
            (
                "close_orderbook",
                build_close_orderbook_ix(
                    &CloseOrderbookParams {
                        operator: signer,
                        orderbook: Pubkey::new_unique(),
                        market,
                    },
                    program_id,
                ),
            ),
        ]
    }

    #[test]
    fn every_public_builder_ends_with_event_transport_trailer() {
        let program_id = test_program_id();
        let expected = event_transport_trailer(&program_id);
        let built = all_public_builders(&program_id);
        assert_eq!(
            built.len(),
            36,
            "register new builders in all_public_builders"
        );

        // Discriminator 6 (the retired IncrementNonce) is not a public instruction.
        let actual_ids: std::collections::BTreeSet<_> =
            built.iter().map(|(_, ix)| ix.data[0]).collect();
        let expected_ids: std::collections::BTreeSet<_> = (0u8..=38)
            .filter(|id| ![6, 21, 23, 26, 34].contains(id))
            .collect();
        assert_eq!(actual_ids, expected_ids);

        for (name, ix) in built {
            assert_eq!(ix.program_id, program_id, "{name} program id");
            let (body, trailer) = ix.accounts.split_at(ix.accounts.len() - 2);
            assert_eq!(
                trailer, &expected,
                "{name} must end with [event_authority, program]"
            );
            assert!(
                trailer
                    .iter()
                    .all(|meta| !meta.is_signer && !meta.is_writable),
                "{name} trailer must be read-only and unsigned"
            );
            assert!(
                body.iter().all(|meta| meta.pubkey != expected[0].pubkey),
                "{name} lists the event authority before the trailer"
            );
        }
    }

    #[test]
    fn test_set_fee_receiver_with_atas_keeps_trailer_after_optional_block() {
        let program_id = test_program_id();
        let quote_mints = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let params = SetFeeReceiverWithAtasParams {
            authority: Pubkey::new_unique(),
            new_fee_receiver: Pubkey::new_unique(),
            quote_mints: quote_mints.clone(),
        };

        let ix = build_set_fee_receiver_with_atas_ix(&params, &program_id).unwrap();

        let trailer_start = ix.accounts.len() - 2;
        assert_eq!(
            &ix.accounts[trailer_start..],
            &event_transport_trailer(&program_id)
        );
        assert_eq!(
            ix.accounts[trailer_start - 1].pubkey,
            get_conditional_token_ata(&params.new_fee_receiver, &quote_mints[1])
        );
    }

    #[test]
    fn create_market_and_set_oracle_reject_zero_and_pda_oracles() {
        let program_id = test_program_id();
        let (pda, _) = get_exchange_pda(&program_id);
        for oracle in [Pubkey::default(), pda] {
            assert!(matches!(
                build_create_market_ix(
                    &CreateMarketParams {
                        manager: pda,
                        oracle,
                        num_outcomes: 2,
                        question_id: [0; 32],
                        maker_fee_bps: 0,
                        taker_fee_bps: 0,
                    },
                    0,
                    &program_id
                ),
                Err(SdkError::InvalidOracle)
            ));
            assert!(matches!(
                build_set_oracle_ix(
                    &SetOracleParams {
                        authority: pda,
                        market: pda,
                        new_oracle: oracle,
                    },
                    &program_id
                ),
                Err(SdkError::InvalidOracle)
            ));
        }
        let oracle = *crate::program::constants::INITIALIZE_AUTHORITY;
        let ix = build_set_oracle_ix(
            &SetOracleParams {
                authority: pda,
                market: pda,
                new_oracle: oracle,
            },
            &program_id,
        )
        .unwrap();
        assert_eq!(&ix.data[1..], oracle.as_ref());
        assert_eq!(ix.accounts[0], signer(pda));
        assert_eq!(
            build_set_paused_ix(&pda, true, &program_id).accounts[0],
            signer_mut(pda)
        );
    }

    // ========================================================================
    // Known-answer vectors from the program team's client
    // ========================================================================
    //
    // Every expected account list and data buffer below was generated by
    // `lightcone-client` 0.4.0 at lightcone-pinnochio f1092ae from the same
    // fixed inputs. That client derives addresses and encodes payloads without
    // this SDK, so these tests do not compare the SDK with itself. Account
    // labels name collaterals A = [0xa1; 32], B = [0x3c; 32], G0 = [0x40; 32]
    // and G1 = [0x41; 32]; a digit after a collateral is the conditional
    // mint's outcome.

    /// The local deployment the client vectors were generated for.
    const CLIENT_PROGRAM_ID: &str = "Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai";

    /// Fixed inputs shared by the client vectors.
    struct ClientInputs {
        program_id: Pubkey,
        market: Pubkey,
        collateral_a: Pubkey,
        collateral_b: Pubkey,
        operator: Pubkey,
        fee_receiver: Pubkey,
        manager: Pubkey,
        user: Pubkey,
    }

    impl ClientInputs {
        /// The conditional mint for one collateral and outcome.
        fn mint(&self, collateral: &Pubkey, outcome: u8) -> Pubkey {
            get_conditional_mint_pda(&self.market, collateral, outcome, &self.program_id).0
        }

        /// Unsigned order template on one pair. Callers set every order term.
        fn pair(&self, base_mint: Pubkey, quote_mint: Pubkey) -> OrderPayload {
            OrderPayload {
                salt: 0,
                maker: Pubkey::default(),
                market: self.market,
                base_mint,
                quote_mint,
                side: OrderSide::Bid,
                amount_in: 0,
                amount_out: 0,
                expiration: 0,
                signature: [0; 64],
            }
        }
    }

    /// Deterministic Ed25519 wallet, so order signatures are reproducible.
    fn wallet(seed: u8) -> solana_keypair::Keypair {
        solana_keypair::Keypair::new_from_array([seed; 32])
    }

    fn client_inputs() -> ClientInputs {
        use solana_signer::Signer;
        ClientInputs {
            program_id: CLIENT_PROGRAM_ID.parse().unwrap(),
            market: Pubkey::new_from_array([0x4d; 32]),
            collateral_a: Pubkey::new_from_array([0xa1; 32]),
            collateral_b: Pubkey::new_from_array([0x3c; 32]),
            operator: wallet(1).pubkey(),
            fee_receiver: wallet(2).pubkey(),
            manager: wallet(6).pubkey(),
            user: wallet(42).pubkey(),
        }
    }

    /// Make wallet `seed` the maker and sign the order-id hex, as the client does.
    fn signed_by(seed: u8, mut order: OrderPayload) -> OrderPayload {
        use solana_signer::Signer;
        let wallet = wallet(seed);
        order.maker = wallet.pubkey();
        let signature = wallet.sign_message(order.hash_hex().as_bytes());
        order.signature = signature.into();
        order
    }

    /// Assert the program id, every account meta in order, and the data bytes.
    fn assert_ix(
        ix: &Instruction,
        program_id: &Pubkey,
        accounts: &[(&str, bool, bool)],
        data_hex: &str,
    ) {
        assert_eq!(ix.program_id, *program_id, "program id");
        assert_eq!(ix.accounts.len(), accounts.len(), "account count");
        for (index, (actual, (pubkey, is_signer, is_writable))) in
            ix.accounts.iter().zip(accounts).enumerate()
        {
            assert_eq!(
                (
                    actual.pubkey.to_string(),
                    actual.is_signer,
                    actual.is_writable
                ),
                (pubkey.to_string(), *is_signer, *is_writable),
                "account {index}"
            );
        }
        assert_eq!(hex::encode(&ix.data), data_hex, "data");
    }

    /// Two-maker MatchOrdersMulti on the A0/B0 book (base mint sorts first).
    /// The BUY taker keeps its status account, maker 0 fully fills its base
    /// and omits its status (mask bit 0), and maker 1 fills partially.
    fn client_match_params(inputs: &ClientInputs) -> MatchOrdersMultiParams {
        let pair = inputs.pair(
            inputs.mint(&inputs.collateral_a, 0),
            inputs.mint(&inputs.collateral_b, 0),
        );
        let taker = signed_by(
            100,
            OrderPayload {
                salt: 0xdead_beef_0000_0001,
                side: OrderSide::Bid,
                amount_in: 120,
                amount_out: 200,
                expiration: 0,
                ..pair.clone()
            },
        );
        let maker_0 = signed_by(
            10,
            OrderPayload {
                salt: u64::MAX,
                side: OrderSide::Ask,
                amount_in: 100,
                amount_out: 60,
                expiration: 1_900_000_000,
                ..pair.clone()
            },
        );
        let maker_1 = signed_by(
            11,
            OrderPayload {
                salt: 7,
                side: OrderSide::Ask,
                amount_in: 150,
                amount_out: 90,
                expiration: i64::MAX,
                ..pair.clone()
            },
        );
        MatchOrdersMultiParams {
            operator: inputs.operator,
            market: inputs.market,
            base_mint: pair.base_mint,
            quote_mint: pair.quote_mint,
            base_deposit_mint: inputs.collateral_a,
            quote_deposit_mint: inputs.collateral_b,
            fee_receiver: inputs.fee_receiver,
            taker_order: taker,
            maker_orders: vec![maker_0, maker_1],
            // Each maker gives 100 base atoms for 60 quote atoms.
            maker_fill_amounts: vec![100, 100],
            taker_fill_amounts: vec![60, 60],
            full_fill_bitmask: 0x0001,
        }
    }

    /// Single-maker DepositAndSwap on the A1/B1 book, whose base mint sorts
    /// after its quote mint, so GDT B precedes GDT A. The SELL taker fully
    /// fills (status omitted) from a collateral-A global deposit; the partial
    /// BUY maker keeps its status and deposits collateral B.
    fn client_deposit_and_swap_params(inputs: &ClientInputs) -> DepositAndSwapParams {
        let pair = inputs.pair(
            inputs.mint(&inputs.collateral_a, 1),
            inputs.mint(&inputs.collateral_b, 1),
        );
        let taker = signed_by(
            101,
            OrderPayload {
                salt: 0,
                side: OrderSide::Ask,
                amount_in: 100,
                amount_out: 60,
                expiration: 0,
                ..pair.clone()
            },
        );
        let maker = signed_by(
            12,
            OrderPayload {
                salt: 0x1112_1314_1516_1718,
                side: OrderSide::Bid,
                amount_in: 90,
                amount_out: 150,
                expiration: 1_700_000_000,
                ..pair.clone()
            },
        );
        DepositAndSwapParams {
            operator: inputs.operator,
            market: inputs.market,
            base_mint: pair.base_mint,
            quote_mint: pair.quote_mint,
            base_deposit_mint: inputs.collateral_a,
            quote_deposit_mint: inputs.collateral_b,
            fee_receiver: inputs.fee_receiver,
            taker_order: taker,
            taker_is_full_fill: true,
            taker_is_deposit: true,
            taker_deposit_mint: inputs.collateral_a,
            num_outcomes: 2,
            // The maker gives 60 quote atoms for 100 base atoms.
            makers: vec![MakerFill {
                order: maker,
                maker_fill_amount: 60,
                taker_fill_amount: 100,
                is_full_fill: false,
                is_deposit: true,
                deposit_mint: inputs.collateral_b,
            }],
        }
    }

    #[test]
    fn mint_complete_set_matches_program_client() {
        let inputs = client_inputs();
        let ix = build_deposit_ix(
            &BuildDepositParams {
                user: inputs.user,
                market: inputs.market,
                deposit_mint: inputs.collateral_a,
                amount: 123_456_789_012,
            },
            2,
            &inputs.program_id,
        );

        // Generated by lightcone-client at lightcone-pinnochio f1092ae.
        assert_ix(
            &ix,
            &inputs.program_id,
            &[
                ("2iXtA8oeZqUU5pofxK971TCEvFGfems2AcDRaZHKD2pQ", true, true), // user
                ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", false, false), // exchange
                ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", false, false), // market
                ("Bswb3UyeD1pUTaGiE6WvqwFpJZsQSEY1xhJePCDTHdvp", false, false), // collateral A
                ("CcJ9vwR45b6DNPChMaS8JG3ua6VhyrGU1zAmSvvkhaRD", false, true), // vault A
                ("FCvj66xiYGJ6SMsZ91UUhN4xvS4Hu7ouRdwzMeECKWjL", false, true), // user collateral A ATA
                ("4CzHZWutNwxxUyV3TigWtWZhBqmRibWuzZqYNvAwYV1p", false, true), // user position
                ("HwerbZ3uDMyHrs1xe1gUfKBHDbmqp4iMcBzgGDNgKPZG", false, false), // mint authority
                ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", false, false), // token program
                ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", false, false), // ATA program
                ("11111111111111111111111111111111", false, false),            // system program
                ("6Deb7C6szQAGzATpea8mwdk98yKLShbep7HQaKeZ4sVB", false, true), // mint A0
                ("C2mDQ76H3X73j8sRj8QtZF9RXSyN4VUdJKvGQ6DTKk76", false, true), // position A0 ATA
                ("DxkG46NfTmZdeLf88KacJwSFeEo8jeta4pQLGvstaKsa", false, true), // mint A1
                ("5jgjy7yUFwQUcnDMDXKR3yL9wcGJQdDhE9L3SQ9uCH1n", false, true), // position A1 ATA
                ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", false, false), // event authority
                ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", false, false), // program
            ],
            "03141a99be1c000000",
        );
    }

    #[test]
    fn create_orderbook_canonicalizes_both_mint_orders_like_program_client() {
        let inputs = client_inputs();
        let (a, b) = (inputs.collateral_a, inputs.collateral_b);
        let build = |mint_a, mint_a_deposit_mint, mint_b, mint_b_deposit_mint, base_index| {
            build_create_orderbook_ix(
                &CreateOrderbookParams {
                    manager: inputs.manager,
                    market: inputs.market,
                    mint_a,
                    mint_b,
                    fee_receiver: inputs.fee_receiver,
                    mint_a_deposit_mint,
                    mint_b_deposit_mint,
                    base_index,
                    outcome_index: 1,
                },
                &inputs.program_id,
            )
            .unwrap()
        };
        let (mint_a1, mint_b1) = (inputs.mint(&a, 1), inputs.mint(&b, 1));

        // Generated by lightcone-client at lightcone-pinnochio f1092ae for the
        // two outcome-1 books: base A1/quote B1 and base B1/quote A1. Mint B1
        // sorts first, so the books share every account except the fee
        // receiver's quote ATA, and only the base index differs in the data.
        let accounts = |fee_receiver_quote_ata| {
            [
                ("AKkzLhjhyFtM9j7WAhbaqYpFe49cXeJBg2kzLRC2PnNa", true, true), // manager
                ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", false, false), // market
                ("9Vm6zp8LJ7BLKA5WXMCrVp1XtPcucDxGSniwcibC4qnY", false, false), // mint B1
                ("DxkG46NfTmZdeLf88KacJwSFeEo8jeta4pQLGvstaKsa", false, false), // mint A1
                ("E11bpYzs4qWFn7yiN2rqwnhCidnUFvwxvDmAgwWaYznw", false, true), // orderbook
                ("7GHv8nJvVXunMXBpQF5t1EbDJtfLxKqXAjLCAtfrzgmf", false, false), // GDT B
                ("GjYEnbey6TFjN6wSf6ZVWzPSykh9GPVBAo26oiXC7KhY", false, false), // GDT A
                ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", false, false), // exchange
                ("11111111111111111111111111111111", false, false),           // system program
                ("548jb4wcUtpbNS1TAva8TXJmeXd5QpCCXUzNVogwpzMR", false, false), // collateral B
                ("Bswb3UyeD1pUTaGiE6WvqwFpJZsQSEY1xhJePCDTHdvp", false, false), // collateral A
                ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", false, false), // token program
                ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", false, false), // ATA program
                ("9hSR6S7WPtxmTojgo6GG3k4yDPecgJY292j7xrsUGWBu", false, false), // fee receiver
                (fee_receiver_quote_ata, false, true),
                ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", false, false), // event authority
                ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", false, false), // program
            ]
        };
        // Fee receiver ATAs for quote mints B1 and A1.
        let base_a1 = accounts("4yzmLNrFRbmMUH9g5HaraxzLHq2CLbh1BWHYcGfesJFL");
        let base_b1 = accounts("73zFJD8Vtz2ash9VDypWXgnToNE5TH2fuT7CygK3ZY5T");

        // Either supplied mint order yields the client's canonical instruction.
        for ix in [
            build(mint_a1, a, mint_b1, b, 0),
            build(mint_b1, b, mint_a1, a, 1),
        ] {
            assert_ix(&ix, &inputs.program_id, &base_a1, "0f0101");
        }
        for ix in [
            build(mint_b1, b, mint_a1, a, 0),
            build(mint_a1, a, mint_b1, b, 1),
        ] {
            assert_ix(&ix, &inputs.program_id, &base_b1, "0f0001");
        }
    }

    #[test]
    fn close_orderbook_matches_program_client() {
        let inputs = client_inputs();
        let (base_mint, quote_mint) = (
            inputs.mint(&inputs.collateral_a, 1),
            inputs.mint(&inputs.collateral_b, 1),
        );
        let ix = build_close_orderbook_ix(
            &CloseOrderbookParams {
                operator: inputs.operator,
                orderbook: get_orderbook_pda(&base_mint, &quote_mint, &inputs.program_id).0,
                market: inputs.market,
            },
            &inputs.program_id,
        );

        // Generated by lightcone-client at lightcone-pinnochio f1092ae.
        assert_ix(
            &ix,
            &inputs.program_id,
            &[
                ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", true, true), // operator
                ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", false, false), // exchange
                ("E11bpYzs4qWFn7yiN2rqwnhCidnUFvwxvDmAgwWaYznw", false, true), // orderbook
                ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", false, false), // market
                ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", false, false), // event authority
                ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", false, false), // program
            ],
            "1b",
        );
    }

    #[test]
    fn global_deposit_builders_match_program_client() {
        let inputs = client_inputs();
        let deposit = build_deposit_to_global_ix(
            &DepositToGlobalParams {
                user: inputs.user,
                mint: inputs.collateral_a,
                amount: u64::MAX - 1,
            },
            &inputs.program_id,
        );
        let to_market = build_global_to_market_deposit_ix(
            &GlobalToMarketDepositParams {
                user: inputs.user,
                market: inputs.market,
                deposit_mint: inputs.collateral_b,
                amount: 500_000,
            },
            2,
            &inputs.program_id,
        );

        // Generated by lightcone-client at lightcone-pinnochio f1092ae.
        assert_ix(
            &deposit,
            &inputs.program_id,
            &[
                ("2iXtA8oeZqUU5pofxK971TCEvFGfems2AcDRaZHKD2pQ", true, true), // user
                ("GjYEnbey6TFjN6wSf6ZVWzPSykh9GPVBAo26oiXC7KhY", false, false), // GDT A
                ("Bswb3UyeD1pUTaGiE6WvqwFpJZsQSEY1xhJePCDTHdvp", false, false), // collateral A
                ("3MWB38PADzqKzPnVTL3Z8YTwyLHpTmKaF3Aj4AU4j8jJ", false, true), // user global deposit A
                ("FCvj66xiYGJ6SMsZ91UUhN4xvS4Hu7ouRdwzMeECKWjL", false, true), // user collateral A ATA
                ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", false, false), // token program
                ("11111111111111111111111111111111", false, false),            // system program
                ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", false, false), // exchange
                ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", false, false), // event authority
                ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", false, false), // program
            ],
            "11feffffffffffffff",
        );
        assert_ix(
            &to_market,
            &inputs.program_id,
            &[
                ("2iXtA8oeZqUU5pofxK971TCEvFGfems2AcDRaZHKD2pQ", true, true), // user
                ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", false, false), // exchange
                ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", false, false), // market
                ("548jb4wcUtpbNS1TAva8TXJmeXd5QpCCXUzNVogwpzMR", false, false), // collateral B
                ("9Ni78X3DJWYiXpymPmA1N8hVdiFPCetaeYsoxaxuF3vz", false, true), // vault B
                ("7GHv8nJvVXunMXBpQF5t1EbDJtfLxKqXAjLCAtfrzgmf", false, false), // GDT B
                ("EBijBMb6kV1e7jNpjCqgQuAQgX9oefqTBTHu4M4qtj1d", false, true), // user global deposit B
                ("4CzHZWutNwxxUyV3TigWtWZhBqmRibWuzZqYNvAwYV1p", false, true), // user position
                ("HwerbZ3uDMyHrs1xe1gUfKBHDbmqp4iMcBzgGDNgKPZG", false, false), // mint authority
                ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", false, false), // token program
                ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", false, false), // ATA program
                ("11111111111111111111111111111111", false, false),            // system program
                ("CTyZDSYMyGRFE2XoFxproKZqrRe6RyVSghupF1dznVmh", false, true), // mint B0
                ("FZoibJ9Bek51d82eKoryEo6GuW9nmYaPicZym3RawkXC", false, true), // position B0 ATA
                ("9Vm6zp8LJ7BLKA5WXMCrVp1XtPcucDxGSniwcibC4qnY", false, true), // mint B1
                ("7jjbtKwm4p3gcTT2NYmdnNtfw5vPRRvg9B7nKz19y5g1", false, true), // position B1 ATA
                ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", false, false), // event authority
                ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", false, false), // program
            ],
            "1220a1070000000000",
        );
    }

    #[test]
    fn position_token_account_builders_match_program_client() {
        let inputs = client_inputs();
        // Two groups in registration order (GDT indices 0 and 3 in the client).
        let groups = vec![
            Pubkey::new_from_array([0x40; 32]),
            Pubkey::new_from_array([0x41; 32]),
        ];
        let init = build_init_position_tokens_ix(
            &InitPositionTokensParams {
                payer: inputs.operator,
                user: inputs.user,
                market: inputs.market,
                deposit_mints: groups.clone(),
            },
            2,
            &inputs.program_id,
        );
        let close = build_close_position_token_accounts_ix(
            &ClosePositionTokenAccountsParams {
                operator: inputs.operator,
                market: inputs.market,
                position: get_position_pda(&inputs.user, &inputs.market, &inputs.program_id).0,
                deposit_mints: groups,
            },
            2,
            &inputs.program_id,
        )
        .unwrap();

        // Generated by lightcone-client at lightcone-pinnochio f1092ae.
        assert_ix(
            &init,
            &inputs.program_id,
            &[
                ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", true, true), // payer
                ("2iXtA8oeZqUU5pofxK971TCEvFGfems2AcDRaZHKD2pQ", false, false), // user
                ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", false, false), // exchange
                ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", false, false), // market
                ("4CzHZWutNwxxUyV3TigWtWZhBqmRibWuzZqYNvAwYV1p", false, true), // user position
                ("HwerbZ3uDMyHrs1xe1gUfKBHDbmqp4iMcBzgGDNgKPZG", false, false), // mint authority
                ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", false, false), // token program
                ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", false, false), // ATA program
                ("11111111111111111111111111111111", false, false),           // system program
                ("5KovAGoer61Vvo1Uv7sod2PpdATt74wUm7ezjKsLpKeF", false, false), // collateral G0
                ("FPaub4kYnsuF2nhEzkGrzruZwGU7qKQGAcSKaisjsd31", false, false), // vault G0
                ("G8KpaNysbSRbPd2oKtuwnqpaz9yNhbDCtqXqdaLfPNZC", false, false), // GDT G0
                ("7xiscy1E21eAJN9B8jL8KcJL5vSBEFoYfBowvAUBxLn5", false, false), // mint G00
                ("Ap7pkFebmQzwJNuhtce4ps7yqoq2E2RPy3GxAfqZXdqP", false, true), // position G00 ATA
                ("DJgBqQz9rCFoztQUBRUv5iXYUahaQfN8Rwd6PcUf8FqV", false, false), // mint G01
                ("63GjdPDfRJWzbbxnU1nXXEfZLq2DEotUkxLC735FMZYC", false, true), // position G01 ATA
                ("5PjDJaGfSPJj4tFzMRCiuuAasKg5n8dJKXKenhuwZexx", false, false), // collateral G1
                ("3vmGXu4dC4zTJQybJsBfpcxqqTcqEUM5hHN5Nr7w46yj", false, false), // vault G1
                ("2bzkpDhvuTJxZCPgmqwwrx3uezdLZEidQPdpLPeqYvoB", false, false), // GDT G1
                ("9KH9qQqJ3Poka7oKTyd2ejDwYfjk5U5nzkfjjNxM2BaZ", false, false), // mint G10
                ("DDPLSXHzmcSEKyS5HAzQZtKZkmC2vg6cvYATtRuBePZ9", false, true), // position G10 ATA
                ("6UMX23ZUbMDeH3S9FHW3thYPvTowzMDUrQsVsjb54zxF", false, false), // mint G11
                ("Ain499vM3dfWTeRaqNdpr5dxfXPiBXAL4VPVWSj4wSSq", false, true), // position G11 ATA
                ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", false, false), // event authority
                ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", false, false), // program
            ],
            "1302",
        );
        assert_ix(
            &close,
            &inputs.program_id,
            &[
                ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", true, true), // operator
                ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", false, false), // exchange
                ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", false, false), // market
                ("4CzHZWutNwxxUyV3TigWtWZhBqmRibWuzZqYNvAwYV1p", false, false), // user position
                ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", false, false), // token program
                ("5KovAGoer61Vvo1Uv7sod2PpdATt74wUm7ezjKsLpKeF", false, false), // collateral G0
                ("7xiscy1E21eAJN9B8jL8KcJL5vSBEFoYfBowvAUBxLn5", false, false), // mint G00
                ("Ap7pkFebmQzwJNuhtce4ps7yqoq2E2RPy3GxAfqZXdqP", false, true), // position G00 ATA
                ("DJgBqQz9rCFoztQUBRUv5iXYUahaQfN8Rwd6PcUf8FqV", false, false), // mint G01
                ("63GjdPDfRJWzbbxnU1nXXEfZLq2DEotUkxLC735FMZYC", false, true), // position G01 ATA
                ("5PjDJaGfSPJj4tFzMRCiuuAasKg5n8dJKXKenhuwZexx", false, false), // collateral G1
                ("9KH9qQqJ3Poka7oKTyd2ejDwYfjk5U5nzkfjjNxM2BaZ", false, false), // mint G10
                ("DDPLSXHzmcSEKyS5HAzQZtKZkmC2vg6cvYATtRuBePZ9", false, true), // position G10 ATA
                ("6UMX23ZUbMDeH3S9FHW3thYPvTowzMDUrQsVsjb54zxF", false, false), // mint G11
                ("Ain499vM3dfWTeRaqNdpr5dxfXPiBXAL4VPVWSj4wSSq", false, true), // position G11 ATA
                ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", false, false), // event authority
                ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", false, false), // program
            ],
            "19",
        );
    }

    #[test]
    fn cancel_order_matches_program_client() {
        let inputs = client_inputs();
        let pair = inputs.pair(
            inputs.mint(&inputs.collateral_a, 0),
            inputs.mint(&inputs.collateral_b, 0),
        );
        let order = signed_by(
            50,
            OrderPayload {
                salt: 0x0102_0304_0506_0708,
                side: OrderSide::Bid,
                amount_in: 123_456,
                amount_out: 654_321,
                expiration: 1_900_000_000,
                ..pair
            },
        );
        let ix =
            build_cancel_order_ix(&inputs.operator, &inputs.market, &order, &inputs.program_id);

        // Generated by lightcone-client at lightcone-pinnochio f1092ae.
        assert_ix(
            &ix,
            &inputs.program_id,
            &[
                ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", true, true), // operator
                ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", false, false), // exchange
                ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", false, false), // market
                ("CZyZ59NJHBRNoDKfndnNggqYHGbZ3xdDF75ikuHHJxDw", false, true), // order status
                ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", false, false), // event authority
                ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", false, false), // program
            ],
            concat!(
                "05",
                // Order id, then the 225-byte signed order.
                "76c704f365c47163d7443660ada391788ffd280a9c1e0d5a26af5394f25ad043",
                "0807060504030201",
                "5e212c0980e4b39fc09721134aa02109374edfd260c0d3d03cb501c8d65457a9",
                "4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d",
                "4d87e7c084828730227139c14226f040f48038ba06198bfb0ffeb9514acaea02",
                "aa599aca0ddeca9f522c30f07b7c0ba0a4a46c37c10c8373b27ffdfbba4af828",
                "00",
                "40e2010000000000",
                "f1fb090000000000",
                "00b33f7100000000",
                "58fc3a9edb7464fb467a78e4603ae1238953860e55c7ed32f9ba9aa98afea114",
                "578bdeb6f4c35f3b55138f41f6059918a267df3348a9576fb668cd9436530506",
            ),
        );
    }

    #[test]
    fn match_orders_multi_matches_program_client() {
        let inputs = client_inputs();
        let ix =
            build_match_orders_multi_ix(&client_match_params(&inputs), &inputs.program_id).unwrap();

        // Generated by lightcone-client at lightcone-pinnochio f1092ae.
        assert_ix(
            &ix,
            &inputs.program_id,
            &[
                ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", true, true), // operator
                ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", false, false), // exchange
                ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", false, false), // market
                ("AcefzzBzYiYcLKmg9ktNNyBnPSxagC9JRNMweSN1kYF5", false, false), // orderbook
                ("GjYEnbey6TFjN6wSf6ZVWzPSykh9GPVBAo26oiXC7KhY", false, false), // GDT A
                ("7GHv8nJvVXunMXBpQF5t1EbDJtfLxKqXAjLCAtfrzgmf", false, false), // GDT B
                ("D6hX8jPNF1a2ZJApa9duWbGMneXN5XB5wATXTdj5U5Ja", false, true), // taker status
                ("FBTMbey9FRyKTJ8bzGK8wokrwyVXjdnnaaBAiAtHTJCm", false, false), // taker position
                ("6Deb7C6szQAGzATpea8mwdk98yKLShbep7HQaKeZ4sVB", false, false), // mint A0 (base)
                ("CTyZDSYMyGRFE2XoFxproKZqrRe6RyVSghupF1dznVmh", false, false), // mint B0 (quote)
                ("BKUo9SY9CjBD3WycwT96RPHzsqriXsgwZu4JZ31KnKpQ", false, true), // taker A0 ATA
                ("BJTYzqmaZKc8U91F8nfqwfiFb6xGCToLii3mGFh9xbkL", false, true), // taker B0 ATA
                ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", false, false), // token program
                ("11111111111111111111111111111111", false, false),           // system program
                ("6DwXxNbBGFceSZbin2oLSP1DnPSXVD4er4U8ZBVW9Pxq", false, true), // fee receiver B0 ATA
                ("9hSR6S7WPtxmTojgo6GG3k4yDPecgJY292j7xrsUGWBu", false, false), // fee receiver
                ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", false, false), // ATA program
                ("AswUFYjEDQi4cXqmde8M1H4cZRD4BnmvKfbQoTqFBLB2", false, false), // maker 0 position
                ("7tJKnMHYLKz7sk3xyDh41PQZbTV7vmSchtMZJAE7G81j", false, true), // maker 0 A0 ATA
                ("4nV6kAnVDp5xLzzcyqFxpiWknSuF5YFZCtx7kZrr4p9R", false, true), // maker 0 B0 ATA
                ("HbPnPSmp428fK7dnAtKwDdLoFu6mpQdHrz5b85Zyj2Em", false, true), // maker 1 status
                ("DagzdYGfpRUgMmsnYZbzHPea9RkPjqJvj41mSg1Erx7w", false, false), // maker 1 position
                ("8TAgt4BvP2aMFJ5T5or4dxgPmHNuGLx2c5HqSbg1U9vc", false, true), // maker 1 A0 ATA
                ("EFiBM1hPrxAdfzKjZMhiGQDqfHeshPqcN38oiFfD6Twv", false, true), // maker 1 B0 ATA
                ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", false, false), // event authority
                ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", false, false), // program
            ],
            concat!(
                "0d",
                // Taker compact order and signature.
                "01000000efbeadde007800000000000000c8000000000000000000000000000000",
                "65e302ae869d94a86e237bfb06dfa084adefa4f461ad90dce13c12d778ac2c26",
                "c480c307132f45a10ad75cdef22c8206d76397a3c0e737195d720682956b2009",
                // Two makers; full-fill mask 0x0001.
                "02",
                "0100",
                // Maker 0: compact order, signature, maker fill, taker fill.
                "ffffffffffffffff0164000000000000003c0000000000000000b33f7100000000",
                "9b29d9b4d5fbcca95d70100554932d394e3136a883dfd71afe77d7583ffe7b6c",
                "0846996b6082b1426b937eaf07899db41dd1c9b01c8bc180aed3cf7bf0fb7701",
                "6400000000000000",
                "3c00000000000000",
                // Maker 1.
                "07000000000000000196000000000000005a00000000000000ffffffffffffff7f",
                "3f32e60f1428f7eff0611c334bdd0315987680e864ec6c8c0b5622fcac9cd535",
                "e651111c8805b5bdc6c6c128de2f8c25fc13d9f8ad5e86e36117fc971b17740f",
                "6400000000000000",
                "3c00000000000000",
            ),
        );
    }

    #[test]
    fn deposit_and_swap_matches_program_client() {
        let inputs = client_inputs();
        let ix =
            build_deposit_and_swap_ix(&client_deposit_and_swap_params(&inputs), &inputs.program_id)
                .unwrap();

        // Generated by lightcone-client at lightcone-pinnochio f1092ae. The SELL
        // taker receives quote (B1) and gives base (A1); every settlement ATA
        // pair follows that orientation, so the maker's B1 ATA closes its
        // deposit block and then opens its settlement pair.
        assert_ix(
            &ix,
            &inputs.program_id,
            &[
                ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", true, true), // operator
                ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", false, false), // exchange
                ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", false, false), // market
                ("E11bpYzs4qWFn7yiN2rqwnhCidnUFvwxvDmAgwWaYznw", false, false), // orderbook
                ("7GHv8nJvVXunMXBpQF5t1EbDJtfLxKqXAjLCAtfrzgmf", false, false), // GDT B
                ("GjYEnbey6TFjN6wSf6ZVWzPSykh9GPVBAo26oiXC7KhY", false, false), // GDT A
                ("HwerbZ3uDMyHrs1xe1gUfKBHDbmqp4iMcBzgGDNgKPZG", false, false), // mint authority
                ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", false, false), // token program
                ("4yzmLNrFRbmMUH9g5HaraxzLHq2CLbh1BWHYcGfesJFL", false, true), // fee receiver B1 ATA
                ("9hSR6S7WPtxmTojgo6GG3k4yDPecgJY292j7xrsUGWBu", false, false), // fee receiver
                ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", false, false), // ATA program
                ("Gwrh4QtYMroN5j7iuBj8qJ2isAqnM2FRbsw23JaxD3BJ", false, false), // taker position
                ("DxkG46NfTmZdeLf88KacJwSFeEo8jeta4pQLGvstaKsa", false, false), // mint A1 (base)
                ("9Vm6zp8LJ7BLKA5WXMCrVp1XtPcucDxGSniwcibC4qnY", false, false), // mint B1 (quote)
                ("CgaNt6iMtiNk6kCL71sJDDPB3GLFY2b4xNN54KKK97jE", false, true), // taker B1 ATA
                ("FtcEGDzZsu5V3ZoMGiRJf8VscXuWreDoSCFxHRBCc6Xd", false, true), // taker A1 ATA
                ("11111111111111111111111111111111", false, false),            // system program
                ("Bswb3UyeD1pUTaGiE6WvqwFpJZsQSEY1xhJePCDTHdvp", false, false), // collateral A
                ("CcJ9vwR45b6DNPChMaS8JG3ua6VhyrGU1zAmSvvkhaRD", false, true), // vault A
                ("GjYEnbey6TFjN6wSf6ZVWzPSykh9GPVBAo26oiXC7KhY", false, false), // GDT A
                ("G39vUBiTQfZeh61obxAoD6ThrwjANLyVsopaYy45T9Bd", false, true), // taker global deposit A
                ("6Deb7C6szQAGzATpea8mwdk98yKLShbep7HQaKeZ4sVB", false, true), // mint A0
                ("9yFYW3w2B2RyvHgh7KQf7HHqqX3rD2txp32WSwsbB7pW", false, true), // taker A0 ATA
                ("DxkG46NfTmZdeLf88KacJwSFeEo8jeta4pQLGvstaKsa", false, true), // mint A1
                ("FtcEGDzZsu5V3ZoMGiRJf8VscXuWreDoSCFxHRBCc6Xd", false, true), // taker A1 ATA
                ("68cXjdwiAAhMg1i8oAopj6J6j7xZZ2NQi7cES74HUujW", false, true), // maker status
                ("FHyZwUs9jJNaXMzvyWZG6gxhiRqVE4CbxRb92RWHfuxR", false, false), // maker position
                ("548jb4wcUtpbNS1TAva8TXJmeXd5QpCCXUzNVogwpzMR", false, false), // collateral B
                ("9Ni78X3DJWYiXpymPmA1N8hVdiFPCetaeYsoxaxuF3vz", false, true), // vault B
                ("7GHv8nJvVXunMXBpQF5t1EbDJtfLxKqXAjLCAtfrzgmf", false, false), // GDT B
                ("DbQxKggBn8RTGCFTqTkn3qYcMPqEKafwpcitUEZXvVaD", false, true), // maker global deposit B
                ("CTyZDSYMyGRFE2XoFxproKZqrRe6RyVSghupF1dznVmh", false, true), // mint B0
                ("J1NHgXSKBU2wPDptRDnKT2ct163kDxqKAn5rcED6caB3", false, true), // maker B0 ATA
                ("9Vm6zp8LJ7BLKA5WXMCrVp1XtPcucDxGSniwcibC4qnY", false, true), // mint B1
                ("EJKrAAFTCgHsPq3nXLL12Bd8xVDy2P6oN4rEpAfMSuwL", false, true), // maker B1 ATA
                ("EJKrAAFTCgHsPq3nXLL12Bd8xVDy2P6oN4rEpAfMSuwL", false, true), // maker B1 ATA
                ("Fzq9pPAncudmCZsYRQdKsQNtnUFooQwDTeLb7qfQTMfb", false, true), // maker A1 ATA
                ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", false, false), // event authority
                ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", false, false), // program
            ],
            concat!(
                "14",
                // Taker compact order and signature.
                "00000000000000000164000000000000003c000000000000000000000000000000",
                "76d7c446df05eaffbaddfee27a939ab11cd999afc57b80434bc749388567c019",
                "677e9ac826ca8dd1499da1a397e90a4f74c78fb6f4620c1d3c78453052f28d01",
                // One maker; full-fill mask 0x8000; deposit mask 0x8001.
                "01",
                "0080",
                "0180",
                // Maker: compact order, signature, maker fill, taker fill.
                "1817161514131211005a00000000000000960000000000000000f1536500000000",
                "65743720dc9ef272128db330653a217f25daed211354835c0d806cc78f5b539c",
                "7e89332fb9dc1c93b201dbfbefbb50cf900618a24a9444a71c26ff74359b1404",
                "3c00000000000000",
                "6400000000000000",
            ),
        );
    }

    #[test]
    fn match_orders_multi_rejects_invalid_trades_locally() {
        let inputs = client_inputs();
        let valid = client_match_params(&inputs);
        let build = |params: &MatchOrdersMultiParams| {
            build_match_orders_multi_ix(params, &inputs.program_id)
        };

        let mut params = valid.clone();
        params.maker_orders.clear();
        assert!(matches!(
            build(&params),
            Err(SdkError::MissingField(field)) if field == "maker_orders"
        ));

        let mut params = valid.clone();
        params.maker_orders = vec![valid.maker_orders[1].clone(); MAX_MAKERS + 1];
        assert!(matches!(
            build(&params),
            Err(SdkError::TooManyMakers { count: 12 })
        ));

        let mut params = valid.clone();
        params.maker_fill_amounts.pop();
        assert!(matches!(
            build(&params),
            Err(SdkError::MissingField(field)) if field == "maker_fill_amounts"
        ));

        let mut params = valid.clone();
        params.taker_fill_amounts.push(60);
        assert!(matches!(
            build(&params),
            Err(SdkError::MissingField(field)) if field == "taker_fill_amounts"
        ));

        // A participant signed for another pair or market.
        let mut params = valid.clone();
        params.maker_orders[1].quote_mint = inputs.mint(&inputs.collateral_b, 1);
        assert!(matches!(build(&params), Err(SdkError::InvalidOrderbook)));
        let mut params = valid.clone();
        params.taker_order.market = Pubkey::new_from_array([0x4e; 32]);
        assert!(matches!(build(&params), Err(SdkError::InvalidOrderbook)));

        // A maker on the taker's side.
        let mut params = valid.clone();
        params.maker_orders[0].side = OrderSide::Bid;
        assert!(matches!(build(&params), Err(SdkError::InvalidSide(0))));

        // Mask bits beyond the two makers; the taker bit alone stays valid.
        for mask in [0x0004, 0x4000, 0x8004] {
            let mut params = valid.clone();
            params.full_fill_bitmask = mask;
            assert!(matches!(build(&params), Err(SdkError::Serialization(_))));
        }
        let mut params = valid.clone();
        params.full_fill_bitmask = 0x8003;
        assert!(build(&params).is_ok());

        let mut params = valid;
        params.quote_deposit_mint = params.base_deposit_mint;
        assert!(matches!(build(&params), Err(SdkError::DepositMintMismatch)));
    }

    #[test]
    fn deposit_and_swap_rejects_invalid_trades_locally() {
        let inputs = client_inputs();
        let valid = client_deposit_and_swap_params(&inputs);
        let build =
            |params: &DepositAndSwapParams| build_deposit_and_swap_ix(params, &inputs.program_id);

        let mut params = valid.clone();
        params.makers.clear();
        assert!(matches!(
            build(&params),
            Err(SdkError::MissingField(field)) if field == "makers"
        ));

        let mut params = valid.clone();
        params.makers = vec![valid.makers[0].clone(); MAX_MAKERS + 1];
        assert!(matches!(
            build(&params),
            Err(SdkError::TooManyMakers { count: 12 })
        ));

        for num_outcomes in [1, 7] {
            let mut params = valid.clone();
            params.num_outcomes = num_outcomes;
            assert!(matches!(
                build(&params),
                Err(SdkError::InvalidOutcomeCount { count }) if count == num_outcomes
            ));
        }

        // A maker signed for another pair.
        let mut params = valid.clone();
        params.makers[0].order.base_mint = inputs.mint(&inputs.collateral_a, 0);
        assert!(matches!(build(&params), Err(SdkError::InvalidOrderbook)));

        // A maker on the taker's side.
        let mut params = valid.clone();
        params.makers[0].order.side = OrderSide::Ask;
        assert!(matches!(build(&params), Err(SdkError::InvalidSide(1))));

        let mut params = valid.clone();
        params.quote_deposit_mint = params.base_deposit_mint;
        assert!(matches!(build(&params), Err(SdkError::DepositMintMismatch)));

        // Depositors must split the collateral behind the asset their order
        // gives: base collateral for the SELL taker, quote for the BUY maker.
        let mut params = valid.clone();
        params.taker_deposit_mint = inputs.collateral_b;
        assert!(matches!(build(&params), Err(SdkError::DepositMintMismatch)));
        let mut params = valid.clone();
        params.makers[0].deposit_mint = inputs.collateral_a;
        assert!(matches!(build(&params), Err(SdkError::DepositMintMismatch)));

        // A non-depositing participant's deposit mint is not encoded or checked.
        let mut params = valid;
        params.makers[0].is_deposit = false;
        params.makers[0].deposit_mint = Pubkey::default();
        assert!(build(&params).is_ok());
    }
}
