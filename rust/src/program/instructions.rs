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
//! state change (on-chain errors 21 and 68). Public instructions must also be
//! transaction-level; invoking one through another program's CPI fails with
//! on-chain error 73. Every builder here appends the trailer through the
//! private `public_instruction` constructor, so it always occupies the last
//! two account slots.

use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

// System program ID
fn system_program_id() -> Pubkey {
    solana_system_interface::program::ID
}

use crate::program::constants::{
    instruction, ALT_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, MAX_DEPOSIT_MINTS_PER_IX, MAX_MAKERS,
    MAX_OUTCOMES, MIN_OUTCOMES, MPL_TOKEN_METADATA_PROGRAM_ID, RENT_SYSVAR_ID, TOKEN_PROGRAM_ID,
};
use crate::program::error::{SdkError, SdkResult};
use crate::program::orders::OrderPayload;
use crate::program::pda::{
    get_alt_pda, get_condition_tombstone_pda, get_conditional_mint_pda, get_event_authority_pda,
    get_exchange_pda, get_global_deposit_token_pda, get_market_pda, get_mint_authority_pda,
    get_mpl_metadata_pda, get_order_status_pda, get_orderbook_pda, get_position_alt_pda,
    get_position_pda, get_user_global_deposit_pda, get_user_nonce_pda, get_vault_pda,
};
use crate::program::types::{
    AcceptRoleParams, ActivateMarketParams, AddDepositMintParams, BuildDepositParams,
    BuildMergeParams, CloseOrderStatusParams, CloseOrderbookAltParams, CloseOrderbookParams,
    ClosePositionAltParams, ClosePositionTokenAccountsParams, ConditionalMetadataParams,
    CreateMarketParams, CreateOrderbookParams, DepositAndSwapParams, DepositToGlobalAltContext,
    DepositToGlobalParams, ExtendPositionTokensParams, GlobalToMarketDepositParams,
    InitPositionTokensParams, MatchOrdersMultiParams, RedeemWinningsParams,
    RefreshOrderbookAltParams, SetAuthorityParams, SetDepositTokenStatusParams,
    SetFeeReceiverParams, SetFeeReceiverWithAtasParams, SetManagerParams, SetMarketFeesParams,
    SetOracleParams, SettleMarketParams, WhitelistDepositTokenParams,
    WithdrawConditionalFromPositionParams, WithdrawFromGlobalParams, WithdrawFromPositionParams,
};
use crate::program::utils::{
    get_conditional_token_ata, get_deposit_token_ata, serialize_conditional_metadata,
    validate_fee_pair, validate_outcome_count,
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
/// Creates a new market in Pending status.
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

    // Data: [discriminator(1), order_hash(32), OrderPayload(233)] = 266 bytes
    let mut data = Vec::with_capacity(266);
    data.push(instruction::CANCEL_ORDER);
    data.extend_from_slice(&order_hash);
    data.extend_from_slice(&order.serialize());

    public_instruction(program_id, keys, data)
}

/// Build IncrementNonce instruction.
///
/// Increments user's nonce for replay protection / mass cancellation.
pub fn build_increment_nonce_ix(user: &Pubkey, program_id: &Pubkey) -> Instruction {
    let (user_nonce, _) = get_user_nonce_pda(user, program_id);
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![
        signer_mut(*user),
        writable(user_nonce),
        readonly(system_program_id()),
        readonly(exchange),
    ];

    let data = vec![instruction::INCREMENT_NONCE];

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

    let (exchange, _) = get_exchange_pda(program_id);
    let (orderbook, _) = get_orderbook_pda(&params.base_mint, &params.quote_mint, program_id);
    let taker_order_hash = params.taker_order.hash();
    let (taker_nonce, _) = get_user_nonce_pda(&params.taker_order.maker, program_id);
    let (taker_position, _) =
        get_position_pda(&params.taker_order.maker, &params.market, program_id);
    let taker_base_ata = get_conditional_token_ata(&taker_position, &params.base_mint);
    let taker_quote_ata = get_conditional_token_ata(&taker_position, &params.quote_mint);
    let fee_receiver_quote_ata =
        get_conditional_token_ata(&params.fee_receiver, &params.quote_mint);

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
    keys.push(readonly(TOKEN_PROGRAM_ID));
    keys.push(readonly(system_program_id()));
    keys.push(writable(fee_receiver_quote_ata));
    keys.push(readonly(params.fee_receiver));
    keys.push(readonly(ASSOCIATED_TOKEN_PROGRAM_ID));

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

    Ok(public_instruction(program_id, keys, data))
}

/// Build CreateOrderbook instruction.
///
/// Creates an on-chain orderbook with address lookup table.
/// Manager-only — must be called by the exchange manager.
///
/// Accounts (17):
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
/// 11. token_program
/// 12. associated_token_program
/// 13. fee_receiver
/// 14. fee_receiver_quote_ata
/// 15. event_authority (readonly) - Event transport trailer
/// 16. program (readonly) - Event transport trailer
pub fn build_create_orderbook_ix(
    params: &CreateOrderbookParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    let canonical = CanonicalOrderbookMints::from_params(params)?;
    let (exchange, _) = get_exchange_pda(program_id);
    let (orderbook, _) =
        get_orderbook_pda(&canonical.mint_a.mint, &canonical.mint_b.mint, program_id);
    let (lookup_table, _) = get_alt_pda(&orderbook, params.recent_slot);
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
        writable(lookup_table),
        readonly(exchange),
        readonly(*ALT_PROGRAM_ID),
        readonly(system_program_id()),
        readonly(canonical.mint_a.deposit_mint),
        readonly(canonical.mint_b.deposit_mint),
        readonly(TOKEN_PROGRAM_ID),
        readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
        readonly(params.fee_receiver),
        writable(fee_receiver_quote_ata),
    ];

    // Data: [discriminator(1), recent_slot(8), base_index(1), mint_a_outcome_index(1), mint_b_outcome_index(1)] = 12 bytes
    let mut data = Vec::with_capacity(12);
    data.push(instruction::CREATE_ORDERBOOK);
    data.extend_from_slice(&params.recent_slot.to_le_bytes());
    data.push(canonical.base_index());
    data.push(canonical.mint_a.outcome_index);
    data.push(canonical.mint_b.outcome_index);

    Ok(public_instruction(program_id, keys, data))
}

/// Build RefreshOrderbookAlt instruction.
///
/// Manager-only. Ensures the current fee receiver quote ATA exists and appends
/// it to the orderbook ALT when absent. This does not fully reshape old ALTs.
pub fn build_refresh_orderbook_alt_ix(
    params: &RefreshOrderbookAltParams,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);
    let fee_receiver_quote_ata =
        get_conditional_token_ata(&params.fee_receiver, &params.quote_mint);

    let keys = vec![
        signer_mut(params.manager),
        readonly(exchange),
        readonly(params.market),
        readonly(params.orderbook),
        writable(params.lookup_table),
        readonly(params.quote_mint),
        readonly(params.fee_receiver),
        writable(fee_receiver_quote_ata),
        readonly(TOKEN_PROGRAM_ID),
        readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
        readonly(*ALT_PROGRAM_ID),
        readonly(system_program_id()),
    ];

    public_instruction(program_id, keys, vec![instruction::REFRESH_ORDERBOOK_ALT])
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
/// or cancelled. The market condition ID is not changed by the program.
pub fn build_set_oracle_ix(
    params: &SetOracleParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.new_oracle == zero_pubkey() {
        return Err(SdkError::InvalidOracle);
    }

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
/// 1. exchange (readonly) - Exchange PDA
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
/// Manager-only. Updates the backend-visible active flag on a whitelisted
/// GlobalDepositToken. Current on-chain user flows do not gate on this flag.
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
/// Accounts (10, or 13 with an ALT context):
/// 0. user (signer, mut)
/// 1. global_deposit_token (readonly) - Whitelist PDA
/// 2. mint (readonly)
/// 3. user_global_deposit (mut) - User's deposit PDA
/// 4. user_token_account (mut) - User's source token account
/// 5. token_program (readonly)
/// 6. system_program (readonly)
/// 7. exchange (readonly) - Exchange PDA for pause validation
/// + optional ALT block (user_nonce, lookup_table, alt_program) when an ALT context is supplied
/// + event_authority (readonly), program (readonly) - Event transport trailer (always last)
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

/// Build InitPositionTokens instruction.
///
/// Create position, all conditional token ATAs, and an Address Lookup Table.
/// Permissionless — anyone (e.g., backend operator) can pay.
///
/// Idempotent: replaying with the same `recent_slot` reuses the existing table,
/// skips canonical groups already present, and recreates missing token
/// accounts. The table address derives from `(position, recent_slot)`, so a
/// retry must reuse the original slot. The program accepts at most
/// `MAX_DEPOSIT_MINTS_PER_IX` deposit-mint groups per call.
///
/// Accounts (13 + per deposit_mint: 3 + num_outcomes*2):
/// 0. payer (signer, mut) - Pays for account creation
/// 1. user (readonly) - Position owner
/// 2. exchange (readonly)
/// 3. market (readonly)
/// 4. position (mut)
/// 5. lookup_table (mut)
/// 6. mint_authority (readonly)
/// 7. token_program (readonly)
/// 8. ata_program (readonly)
/// 9. alt_program (readonly)
/// 10. system_program (readonly)
/// + per deposit_mint: deposit_mint, vault, gdt, [cond_mint, ata] × num_outcomes
/// + event_authority (readonly), program (readonly) - Event transport trailer
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
        readonly(TOKEN_PROGRAM_ID),
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

    public_instruction(program_id, keys, data)
}

/// Build DepositAndSwap instruction.
///
/// Unified order execution: participants can deposit from global deposits and/or swap
/// conditional tokens in a single instruction. Each participant's deposit is conditional
/// on the deposit_bitmask.
///
/// Account layout:
///   Fixed (9): operator, exchange, market, orderbook, mint_authority, token_program,
///              fee_receiver_quote_ata, fee_receiver, ata_program
///   Taker block: [order_status], nonce, position, base_mint, quote_mint,
///                taker_receive_ata, taker_give_ata, system_program
///   Taker deposit block (optional): deposit_mint, vault, gdt, user_global_deposit,
///                                    [cond_mint, ata] × num_outcomes
///   Per-maker blocks: [order_status], nonce, position,
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

    let (exchange, _) = get_exchange_pda(program_id);
    let (orderbook, _) = get_orderbook_pda(&params.base_mint, &params.quote_mint, program_id);
    let (mint_authority, _) = get_mint_authority_pda(&params.market, program_id);
    let (taker_position, _) =
        get_position_pda(&params.taker_order.maker, &params.market, program_id);
    let (taker_nonce, _) = get_user_nonce_pda(&params.taker_order.maker, program_id);
    let fee_receiver_quote_ata =
        get_conditional_token_ata(&params.fee_receiver, &params.quote_mint);

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

    // Fixed accounts (9)
    keys.push(signer_mut(params.operator));
    keys.push(readonly(exchange));
    keys.push(readonly(params.market));
    keys.push(readonly(orderbook));
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
    keys.push(readonly(taker_nonce));
    keys.push(writable(taker_position));
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

    Ok(public_instruction(program_id, keys, data))
}

/// Build ExtendPositionTokens instruction.
///
/// Extend an existing position ALT with entries for additional deposit mints.
/// Permissionless: any signer may pay. The position PDA remains the table
/// authority, and the program skips canonical groups already present in the
/// table, so callers may pass existing and new mints together. The program
/// rejects a deactivated table, a table with a foreign authority, and a request
/// that would exceed the table's 256-entry capacity.
///
/// Accounts (12 + per deposit_mint: 3 + num_outcomes*2):
/// 0. payer (signer, mut)
/// 1. user (readonly) - Position owner
/// 2. exchange (readonly)
/// 3. market (readonly)
/// 4. position (readonly) - Existing Position PDA
/// 5. lookup_table (mut) - Existing ALT (authority = position PDA)
/// 6. token_program (readonly)
/// 7. ata_program (readonly)
/// 8. alt_program (readonly)
/// 9. system_program (readonly)
/// + per deposit_mint: deposit_mint, vault, global_deposit_token,
///   then per outcome: conditional_mint, position_conditional_ata
/// + event_authority (readonly), program (readonly) - Event transport trailer
pub fn build_extend_position_tokens_ix(
    params: &ExtendPositionTokensParams,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.deposit_mints.is_empty() {
        return Err(SdkError::MissingField("deposit_mints".to_string()));
    }
    if params.deposit_mints.len() > MAX_DEPOSIT_MINTS_PER_IX {
        return Err(SdkError::TooManyDepositMints {
            count: params.deposit_mints.len(),
        });
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let (position, _) = get_position_pda(&params.user, &params.market, program_id);

    let mut keys = vec![
        signer_mut(params.payer),
        readonly(params.user),
        readonly(exchange),
        readonly(params.market),
        readonly(position),
        writable(params.lookup_table),
        readonly(TOKEN_PROGRAM_ID),
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

/// Build ClosePositionAlt instruction.
///
/// Deactivates an active position ALT, or closes an already-deactivated ALT.
pub fn build_close_position_alt_ix(
    params: &ClosePositionAltParams,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![
        signer_mut(params.operator),
        readonly(exchange),
        readonly(params.position),
        readonly(params.market),
        writable(params.lookup_table),
        readonly(*ALT_PROGRAM_ID),
    ];

    public_instruction(program_id, keys, vec![instruction::CLOSE_POSITION_ALT])
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

/// Build CloseOrderbookAlt instruction.
///
/// Deactivates an active orderbook ALT, or closes an already-deactivated ALT.
pub fn build_close_orderbook_alt_ix(
    params: &CloseOrderbookAltParams,
    program_id: &Pubkey,
) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![
        signer_mut(params.operator),
        readonly(exchange),
        readonly(params.orderbook),
        readonly(params.market),
        writable(params.lookup_table),
        readonly(*ALT_PROGRAM_ID),
    ];

    public_instruction(program_id, keys, vec![instruction::CLOSE_ORDERBOOK_ALT])
}

/// Build CloseOrderbook instruction.
///
/// Closes an orderbook PDA after its recorded lookup table has already been
/// closed by the ALT program.
pub fn build_close_orderbook_ix(params: &CloseOrderbookParams, program_id: &Pubkey) -> Instruction {
    let (exchange, _) = get_exchange_pda(program_id);

    let keys = vec![
        signer_mut(params.operator),
        readonly(exchange),
        writable(params.orderbook),
        readonly(params.market),
        readonly(params.lookup_table),
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
    fn test_build_increment_nonce_ix() {
        let user = Pubkey::new_unique();
        let program_id = test_program_id();

        let ix = build_increment_nonce_ix(&user, &program_id);

        assert_eq!(ix.program_id, program_id);
        assert_eq!(ix.accounts.len(), 6);
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
            oracle: Pubkey::new_unique(),
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
            oracle: Pubkey::new_unique(),
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

        assert_eq!(ix.accounts.len(), 6);
        assert_eq!(ix.data.len(), 266); // 1 + 32 + 233
        assert_eq!(ix.data[0], instruction::CANCEL_ORDER);
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
    fn test_build_create_orderbook_ix() {
        let program_id = test_program_id();
        let params = CreateOrderbookParams {
            manager: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            mint_a: Pubkey::new_from_array([2u8; 32]),
            mint_b: Pubkey::new_from_array([1u8; 32]),
            fee_receiver: Pubkey::new_unique(),
            mint_a_deposit_mint: Pubkey::new_from_array([12u8; 32]),
            mint_b_deposit_mint: Pubkey::new_from_array([11u8; 32]),
            recent_slot: 12345,
            base_index: 0,
            mint_a_outcome_index: 2,
            mint_b_outcome_index: 1,
        };

        let ix = build_create_orderbook_ix(&params, &program_id).unwrap();

        assert_eq!(ix.accounts.len(), 17);
        assert_eq!(ix.data.len(), 12); // 1 + 8 + 1 + 1 + 1
        assert_eq!(ix.data[0], instruction::CREATE_ORDERBOOK);
        assert_eq!(ix.accounts[2].pubkey, params.mint_b);
        assert_eq!(ix.accounts[3].pubkey, params.mint_a);
        assert_eq!(ix.accounts[13].pubkey, params.fee_receiver);
        assert_eq!(ix.data[9], 1); // base_index after canonical sorting
        assert_eq!(ix.data[10], 1); // canonical mint_a outcome index
        assert_eq!(ix.data[11], 2); // canonical mint_b outcome index
    }

    #[test]
    fn test_build_refresh_orderbook_alt_ix() {
        let program_id = test_program_id();
        let params = RefreshOrderbookAltParams {
            manager: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            orderbook: Pubkey::new_unique(),
            lookup_table: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            fee_receiver: Pubkey::new_unique(),
        };

        let ix = build_refresh_orderbook_alt_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 14);
        assert_eq!(ix.accounts[0].pubkey, params.manager);
        assert!(ix.accounts[0].is_signer);
        assert!(ix.accounts[0].is_writable);
        assert_eq!(ix.accounts[2].pubkey, params.market);
        assert_eq!(ix.accounts[3].pubkey, params.orderbook);
        assert_eq!(ix.accounts[4].pubkey, params.lookup_table);
        assert!(ix.accounts[4].is_writable);
        assert_eq!(ix.accounts[5].pubkey, params.quote_mint);
        assert_eq!(ix.accounts[6].pubkey, params.fee_receiver);
        assert_eq!(
            ix.accounts[7].pubkey,
            get_conditional_token_ata(&params.fee_receiver, &params.quote_mint)
        );
        assert_eq!(ix.data, vec![instruction::REFRESH_ORDERBOOK_ALT]);
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
            new_oracle: Pubkey::new_unique(),
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
            fee_receiver: Pubkey::new_unique(),
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
        // Taker: 16 accounts, Maker: 5 accounts, trailer: 2 accounts = 23 total
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
            fee_receiver: Pubkey::new_unique(),
            taker_order: taker,
            maker_orders: vec![maker],
            maker_fill_amounts: vec![50],
            taker_fill_amounts: vec![100],
            full_fill_bitmask: 0b10000001,
        };

        let ix = build_match_orders_multi_ix(&params, &program_id).unwrap();

        // With bitmask=0x81 (taker + maker 0 full fill):
        // Taker: 15 accounts (no order_status), Maker: 4 accounts (no order_status),
        // trailer: 2 accounts = 21 total
        assert_eq!(ix.accounts.len(), 21);
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
            user: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            amount: 1_000_000,
        };

        let ix = build_deposit_to_global_ix(&params, &program_id);

        assert_eq!(ix.accounts.len(), 10);
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

        assert_eq!(ix.accounts.len(), 13);
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

        assert_eq!(ix.accounts.len(), 9);
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
            user: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            deposit_mints: vec![deposit_mint],
            recent_slot: 12345,
        };

        let ix = build_init_position_tokens_ix(&params, 3, &program_id);

        // 11 fixed + 1*(3 + 3*2) + 2 trailer = 22
        assert_eq!(ix.accounts.len(), 22);
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
                deposit_mint,
            }],
        };

        let ix = build_deposit_and_swap_ix(&params, &program_id).unwrap();

        // Data: 1 + 37 + 64 + 1 + 1 + 1 + 117 = 222
        assert_eq!(ix.data.len(), 222);
        assert_eq!(ix.data[0], instruction::DEPOSIT_AND_SWAP);

        // Account layout (taker+maker both depositing, no full fills):
        // Fixed: 9
        // Taker order_status: 1
        // Taker common: 7 (nonce, position, base_mint, quote_mint, receive_ata, give_ata, system)
        // Taker deposit: 4 + 3*2 = 10 (dm, vault, gdt, global_deposit, cond_mint+ata*3)
        // Maker order_status: 1
        // Maker common: 2 (nonce, position)
        // Maker deposit: 4 + 3*2 = 10
        // Maker swap: 2 (receive_ata, give_ata)
        // Trailer: 2 (event_authority, program)
        // Total: 9 + 1 + 7 + 10 + 1 + 2 + 10 + 2 + 2 = 44
        assert_eq!(ix.accounts.len(), 44);
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

    #[test]
    fn test_build_close_alt_and_orderbook_ixs() {
        let program_id = test_program_id();
        let operator = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let lookup_table = Pubkey::new_unique();

        let position_alt = build_close_position_alt_ix(
            &ClosePositionAltParams {
                operator,
                position: Pubkey::new_unique(),
                market,
                lookup_table,
            },
            &program_id,
        );
        assert_eq!(position_alt.accounts.len(), 8);
        assert_eq!(position_alt.data, vec![instruction::CLOSE_POSITION_ALT]);

        let orderbook = Pubkey::new_unique();
        let orderbook_alt = build_close_orderbook_alt_ix(
            &CloseOrderbookAltParams {
                operator,
                orderbook,
                market,
                lookup_table,
            },
            &program_id,
        );
        assert_eq!(orderbook_alt.accounts.len(), 8);
        assert_eq!(orderbook_alt.data, vec![instruction::CLOSE_ORDERBOOK_ALT]);

        let close_orderbook = build_close_orderbook_ix(
            &CloseOrderbookParams {
                operator,
                orderbook,
                market,
                lookup_table,
            },
            &program_id,
        );
        assert_eq!(close_orderbook.accounts.len(), 7);
        assert_eq!(close_orderbook.data, vec![instruction::CLOSE_ORDERBOOK]);
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
            nonce: 1,
            salt: 0,
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

    /// One representative instruction per public builder. Register new builders
    /// here so `every_public_builder_ends_with_event_transport_trailer` covers them.
    fn all_public_builders(program_id: &Pubkey) -> Vec<(&'static str, Instruction)> {
        let market = Pubkey::new_unique();
        let deposit_mint = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();
        let signer = Pubkey::new_unique();
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
                        oracle: Pubkey::new_unique(),
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
                "increment_nonce",
                build_increment_nonce_ix(&signer, program_id),
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
                        mint_a_deposit_mint: deposit_mint,
                        mint_b_deposit_mint: deposit_mint,
                        recent_slot: 1,
                        base_index: 0,
                        mint_a_outcome_index: 0,
                        mint_b_outcome_index: 1,
                    },
                    program_id,
                )
                .unwrap(),
            ),
            (
                "refresh_orderbook_alt",
                build_refresh_orderbook_alt_ix(
                    &RefreshOrderbookAltParams {
                        manager: signer,
                        market,
                        orderbook: Pubkey::new_unique(),
                        lookup_table: Pubkey::new_unique(),
                        quote_mint,
                        fee_receiver: Pubkey::new_unique(),
                    },
                    program_id,
                ),
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
                        new_oracle: Pubkey::new_unique(),
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
                "deposit_to_global_with_alt",
                build_deposit_to_global_ix_with_alt(
                    &deposit_to_global,
                    DepositToGlobalAltContext::Extend {
                        lookup_table: Pubkey::new_unique(),
                    },
                    program_id,
                ),
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
                        user: Pubkey::new_unique(),
                        market,
                        deposit_mints: vec![deposit_mint],
                        recent_slot: 1,
                    },
                    2,
                    program_id,
                ),
            ),
            (
                "deposit_and_swap",
                build_deposit_and_swap_ix(
                    &DepositAndSwapParams {
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
                "extend_position_tokens",
                build_extend_position_tokens_ix(
                    &ExtendPositionTokensParams {
                        payer: signer,
                        user: Pubkey::new_unique(),
                        market,
                        lookup_table: Pubkey::new_unique(),
                        deposit_mints: vec![deposit_mint],
                    },
                    2,
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
                "close_position_alt",
                build_close_position_alt_ix(
                    &ClosePositionAltParams {
                        operator: signer,
                        position: Pubkey::new_unique(),
                        market,
                        lookup_table: Pubkey::new_unique(),
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
                "close_orderbook_alt",
                build_close_orderbook_alt_ix(
                    &CloseOrderbookAltParams {
                        operator: signer,
                        orderbook: Pubkey::new_unique(),
                        market,
                        lookup_table: Pubkey::new_unique(),
                    },
                    program_id,
                ),
            ),
            (
                "close_orderbook",
                build_close_orderbook_ix(
                    &CloseOrderbookParams {
                        operator: signer,
                        orderbook: Pubkey::new_unique(),
                        market,
                        lookup_table: Pubkey::new_unique(),
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
            42,
            "register new builders in all_public_builders"
        );

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
    fn test_build_extend_position_tokens_ix_rejects_too_many_groups() {
        let program_id = test_program_id();
        let mut params = ExtendPositionTokensParams {
            payer: Pubkey::new_unique(),
            user: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            lookup_table: Pubkey::new_unique(),
            deposit_mints: (0..=MAX_DEPOSIT_MINTS_PER_IX)
                .map(|_| Pubkey::new_unique())
                .collect(),
        };

        assert!(matches!(
            build_extend_position_tokens_ix(&params, 2, &program_id),
            Err(SdkError::TooManyDepositMints { count }) if count == MAX_DEPOSIT_MINTS_PER_IX + 1
        ));

        params.deposit_mints.truncate(MAX_DEPOSIT_MINTS_PER_IX);
        let ix = build_extend_position_tokens_ix(&params, 2, &program_id).unwrap();
        assert_eq!(ix.accounts[0], signer_mut(params.payer));
    }
}
