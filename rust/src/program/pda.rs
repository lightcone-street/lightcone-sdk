//! PDA (Program Derived Address) derivation functions.
//!
//! This module provides all PDA derivation functions matching the on-chain program.

use solana_sdk::pubkey::Pubkey;

use crate::program::constants::{
    CONDITIONAL_MINT_SEED, EXCHANGE_SEED, MARKET_SEED, MINT_AUTHORITY_SEED, ORDER_STATUS_SEED,
    POSITION_SEED, USER_NONCE_SEED, VAULT_SEED,
};

/// Get the Exchange PDA.
///
/// Seeds: ["central_state"]
pub fn get_exchange_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[EXCHANGE_SEED], program_id)
}

/// Get a Market PDA.
///
/// Seeds: ["market", market_id (8 bytes LE)]
pub fn get_market_pda(market_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MARKET_SEED, &market_id.to_le_bytes()], program_id)
}

/// Get the Vault PDA for a market's deposit mint.
///
/// Seeds: ["market_deposit_token_account", deposit_mint, market]
pub fn get_vault_pda(deposit_mint: &Pubkey, market: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[VAULT_SEED, deposit_mint.as_ref(), market.as_ref()],
        program_id,
    )
}

/// Get the Mint Authority PDA for a market.
///
/// Seeds: ["market_mint_authority", market]
pub fn get_mint_authority_pda(market: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_AUTHORITY_SEED, market.as_ref()], program_id)
}

/// Get a Conditional Mint PDA.
///
/// Seeds: ["conditional_mint", market, deposit_mint, outcome_index (1 byte)]
pub fn get_conditional_mint_pda(
    market: &Pubkey,
    deposit_mint: &Pubkey,
    outcome_index: u8,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CONDITIONAL_MINT_SEED,
            market.as_ref(),
            deposit_mint.as_ref(),
            &[outcome_index],
        ],
        program_id,
    )
}

/// Get all Conditional Mint PDAs for a market.
pub fn get_all_conditional_mint_pdas(
    market: &Pubkey,
    deposit_mint: &Pubkey,
    num_outcomes: u8,
    program_id: &Pubkey,
) -> Vec<(Pubkey, u8)> {
    (0..num_outcomes)
        .map(|i| get_conditional_mint_pda(market, deposit_mint, i, program_id))
        .collect()
}

/// Get an Order Status PDA.
///
/// Seeds: ["order_status", order_hash (32 bytes)]
pub fn get_order_status_pda(order_hash: &[u8; 32], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[ORDER_STATUS_SEED, order_hash], program_id)
}

/// Get a User Nonce PDA.
///
/// Seeds: ["user_nonce", user]
pub fn get_user_nonce_pda(user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[USER_NONCE_SEED, user.as_ref()], program_id)
}

/// Get a Position PDA.
///
/// Seeds: ["position", owner, market]
pub fn get_position_pda(owner: &Pubkey, market: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[POSITION_SEED, owner.as_ref(), market.as_ref()], program_id)
}

/// Collection of all PDA derivation functions for convenient access.
pub struct Pda;

impl Pda {
    /// Get the Exchange PDA.
    pub fn exchange(program_id: &Pubkey) -> (Pubkey, u8) {
        get_exchange_pda(program_id)
    }

    /// Get a Market PDA.
    pub fn market(market_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        get_market_pda(market_id, program_id)
    }

    /// Get the Vault PDA.
    pub fn vault(deposit_mint: &Pubkey, market: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        get_vault_pda(deposit_mint, market, program_id)
    }

    /// Get the Mint Authority PDA.
    pub fn mint_authority(market: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        get_mint_authority_pda(market, program_id)
    }

    /// Get a Conditional Mint PDA.
    pub fn conditional_mint(
        market: &Pubkey,
        deposit_mint: &Pubkey,
        outcome_index: u8,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        get_conditional_mint_pda(market, deposit_mint, outcome_index, program_id)
    }

    /// Get all Conditional Mint PDAs for a market.
    pub fn all_conditional_mints(
        market: &Pubkey,
        deposit_mint: &Pubkey,
        num_outcomes: u8,
        program_id: &Pubkey,
    ) -> Vec<(Pubkey, u8)> {
        get_all_conditional_mint_pdas(market, deposit_mint, num_outcomes, program_id)
    }

    /// Get an Order Status PDA.
    pub fn order_status(order_hash: &[u8; 32], program_id: &Pubkey) -> (Pubkey, u8) {
        get_order_status_pda(order_hash, program_id)
    }

    /// Get a User Nonce PDA.
    pub fn user_nonce(user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        get_user_nonce_pda(user, program_id)
    }

    /// Get a Position PDA.
    pub fn position(owner: &Pubkey, market: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        get_position_pda(owner, market, program_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn test_program_id() -> Pubkey {
        Pubkey::from_str("EfRvELrn4b5aJRwddD1VUrqzsfm1pewBLPebq3iMPDp2").unwrap()
    }

    #[test]
    fn test_exchange_pda_is_deterministic() {
        let program_id = test_program_id();
        let (pda1, bump1) = get_exchange_pda(&program_id);
        let (pda2, bump2) = get_exchange_pda(&program_id);

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn test_market_pda_is_deterministic() {
        let program_id = test_program_id();
        let (pda1, bump1) = get_market_pda(0, &program_id);
        let (pda2, bump2) = get_market_pda(0, &program_id);

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn test_different_market_ids_produce_different_pdas() {
        let program_id = test_program_id();
        let (pda1, _) = get_market_pda(0, &program_id);
        let (pda2, _) = get_market_pda(1, &program_id);

        assert_ne!(pda1, pda2);
    }

    #[test]
    fn test_position_pda_is_deterministic() {
        let program_id = test_program_id();
        let owner = Pubkey::new_unique();
        let market = Pubkey::new_unique();

        let (pda1, bump1) = get_position_pda(&owner, &market, &program_id);
        let (pda2, bump2) = get_position_pda(&owner, &market, &program_id);

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn test_conditional_mint_pdas() {
        let program_id = test_program_id();
        let market = Pubkey::new_unique();
        let deposit_mint = Pubkey::new_unique();

        let pdas = get_all_conditional_mint_pdas(&market, &deposit_mint, 3, &program_id);
        assert_eq!(pdas.len(), 3);

        // All PDAs should be different
        assert_ne!(pdas[0].0, pdas[1].0);
        assert_ne!(pdas[1].0, pdas[2].0);
        assert_ne!(pdas[0].0, pdas[2].0);
    }

    #[test]
    fn test_order_status_pda() {
        let program_id = test_program_id();
        let order_hash = [42u8; 32];

        let (pda1, bump1) = get_order_status_pda(&order_hash, &program_id);
        let (pda2, bump2) = get_order_status_pda(&order_hash, &program_id);

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn test_pda_struct_methods() {
        let program_id = test_program_id();
        let owner = Pubkey::new_unique();
        let market = Pubkey::new_unique();

        // Verify Pda struct methods match the standalone functions
        assert_eq!(
            Pda::exchange(&program_id),
            get_exchange_pda(&program_id)
        );
        assert_eq!(
            Pda::market(5, &program_id),
            get_market_pda(5, &program_id)
        );
        assert_eq!(
            Pda::position(&owner, &market, &program_id),
            get_position_pda(&owner, &market, &program_id)
        );
    }
}
