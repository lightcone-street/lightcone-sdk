//! Instruction builders for all 14 Lightcone Pinocchio instructions.
//!
//! This module provides functions to build transaction instructions for interacting
//! with the Lightcone Pinocchio program.

use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

// System program ID
fn system_program_id() -> Pubkey {
    solana_system_interface::program::ID
}

use crate::constants::{
    instruction, ASSOCIATED_TOKEN_PROGRAM_ID, INSTRUCTIONS_SYSVAR_ID, MAX_MAKERS,
    TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
};
use crate::error::{SdkError, SdkResult};
use crate::orders::FullOrder;
use crate::pda::{
    get_conditional_mint_pda, get_exchange_pda, get_market_pda, get_mint_authority_pda,
    get_order_status_pda, get_position_pda, get_user_nonce_pda, get_vault_pda,
};
use crate::types::{
    ActivateMarketParams, AddDepositMintParams, CreateMarketParams, MatchOrdersMultiParams,
    MergeCompleteSetParams, MintCompleteSetParams, RedeemWinningsParams,
    SettleMarketParams, WithdrawFromPositionParams,
};
use crate::utils::{
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
/// 8+ conditional_mints[0..num_outcomes]
pub fn build_add_deposit_mint_ix(
    params: &AddDepositMintParams,
    market: &Pubkey,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.outcome_metadata.len() != num_outcomes as usize {
        return Err(SdkError::InvalidOutcomeCount(params.outcome_metadata.len() as u8));
    }

    let (vault, _) = get_vault_pda(&params.deposit_mint, market, program_id);
    let (mint_authority, _) = get_mint_authority_pda(market, program_id);

    let mut keys = vec![
        signer_mut(params.payer),
        writable(*market),
        readonly(params.deposit_mint),
        writable(vault),
        readonly(mint_authority),
        readonly(*TOKEN_PROGRAM_ID),
        readonly(*TOKEN_2022_PROGRAM_ID),
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
        readonly(*TOKEN_PROGRAM_ID),
        readonly(*TOKEN_2022_PROGRAM_ID),
        readonly(*ASSOCIATED_TOKEN_PROGRAM_ID),
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
        readonly(*TOKEN_PROGRAM_ID),
        readonly(*TOKEN_2022_PROGRAM_ID),
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
pub fn build_cancel_order_ix(
    maker: &Pubkey,
    order: &FullOrder,
    program_id: &Pubkey,
) -> Instruction {
    let order_hash = order.hash();
    let (order_status, _) = get_order_status_pda(&order_hash, program_id);

    let keys = vec![
        signer_mut(*maker),
        writable(order_status),
        readonly(system_program_id()),
    ];

    // Data: [discriminator, order_hash (32), full_order (225)]
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
        readonly(*TOKEN_PROGRAM_ID),
        readonly(*TOKEN_2022_PROGRAM_ID),
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
        *TOKEN_2022_PROGRAM_ID
    } else {
        *TOKEN_PROGRAM_ID
    };

    let keys = vec![
        signer_mut(params.user),
        writable(position),
        readonly(params.mint),
        writable(position_ata),
        writable(user_ata),
        readonly(token_program),
    ];

    let mut data = Vec::with_capacity(9);
    data.push(instruction::WITHDRAW_FROM_POSITION);
    data.extend_from_slice(&params.amount.to_le_bytes());

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
/// Match taker against up to 5 makers.
///
/// Accounts (13 fixed + 5 per maker):
/// 0. operator (signer)
/// 1. exchange (readonly)
/// 2. market (readonly)
/// 3. taker_order_status (mut)
/// 4. taker_nonce (readonly)
/// 5. taker_position (mut)
/// 6. base_mint (readonly)
/// 7. quote_mint (readonly)
/// 8. taker_position_base_ata (mut)
/// 9. taker_position_quote_ata (mut)
/// 10. token_2022_program (readonly)
/// 11. system_program (readonly)
/// 12. instructions_sysvar (readonly)
/// Per maker: order_status, nonce, position, base_ata, quote_ata
pub fn build_match_orders_multi_ix(
    params: &MatchOrdersMultiParams,
    program_id: &Pubkey,
) -> SdkResult<Instruction> {
    if params.maker_orders.is_empty() {
        return Err(SdkError::MissingField("maker_orders".to_string()));
    }
    if params.maker_orders.len() > MAX_MAKERS {
        return Err(SdkError::TooManyMakers(params.maker_orders.len()));
    }
    if params.maker_orders.len() != params.fill_amounts.len() {
        return Err(SdkError::MissingField("fill_amounts".to_string()));
    }

    let (exchange, _) = get_exchange_pda(program_id);
    let taker_order_hash = params.taker_order.hash();
    let (taker_order_status, _) = get_order_status_pda(&taker_order_hash, program_id);
    let (taker_nonce, _) = get_user_nonce_pda(&params.taker_order.maker, program_id);
    let (taker_position, _) =
        get_position_pda(&params.taker_order.maker, &params.market, program_id);
    let taker_base_ata = get_conditional_token_ata(&taker_position, &params.base_mint);
    let taker_quote_ata = get_conditional_token_ata(&taker_position, &params.quote_mint);

    let mut keys = vec![
        signer_mut(params.operator),
        readonly(exchange),
        readonly(params.market),
        writable(taker_order_status),
        readonly(taker_nonce),
        writable(taker_position),
        readonly(params.base_mint),
        readonly(params.quote_mint),
        writable(taker_base_ata),
        writable(taker_quote_ata),
        readonly(*TOKEN_2022_PROGRAM_ID),
        readonly(system_program_id()),
        readonly(*INSTRUCTIONS_SYSVAR_ID),
    ];

    // Add maker accounts
    for maker_order in &params.maker_orders {
        let maker_order_hash = maker_order.hash();
        let (maker_order_status, _) = get_order_status_pda(&maker_order_hash, program_id);
        let (maker_nonce, _) = get_user_nonce_pda(&maker_order.maker, program_id);
        let (maker_position, _) = get_position_pda(&maker_order.maker, &params.market, program_id);
        let maker_base_ata = get_conditional_token_ata(&maker_position, &params.base_mint);
        let maker_quote_ata = get_conditional_token_ata(&maker_position, &params.quote_mint);

        keys.push(writable(maker_order_status));
        keys.push(readonly(maker_nonce));
        keys.push(writable(maker_position));
        keys.push(writable(maker_base_ata));
        keys.push(writable(maker_quote_ata));
    }

    // Build data
    // [discriminator, taker_hash(32), taker_compact(65), taker_sig(64), num_makers(1)]
    // Per maker: [hash(32), compact(65), sig(64), fill(8)]
    let taker_compact = params.taker_order.to_compact();
    let num_makers = params.maker_orders.len() as u8;

    let data_size = 1 + 32 + 65 + 64 + 1 + (params.maker_orders.len() * (32 + 65 + 64 + 8));
    let mut data = Vec::with_capacity(data_size);

    data.push(instruction::MATCH_ORDERS_MULTI);
    data.extend_from_slice(&taker_order_hash);
    data.extend_from_slice(&taker_compact.serialize());
    data.extend_from_slice(&params.taker_order.signature);
    data.push(num_makers);

    for (i, maker_order) in params.maker_orders.iter().enumerate() {
        let maker_hash = maker_order.hash();
        let maker_compact = maker_order.to_compact();

        data.extend_from_slice(&maker_hash);
        data.extend_from_slice(&maker_compact.serialize());
        data.extend_from_slice(&maker_order.signature);
        data.extend_from_slice(&params.fill_amounts[i].to_le_bytes());
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
    use crate::constants::PROGRAM_ID;

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
}
