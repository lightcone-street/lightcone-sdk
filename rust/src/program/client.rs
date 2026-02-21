//! Async client for interacting with the Lightcone Pinocchio program.
//!
//! This module provides the main SDK client with account fetching and
//! transaction building capabilities.

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_hash::Hash;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;

#[cfg(feature = "native-client")]
use solana_keypair::Keypair;

use crate::program::accounts::{Exchange, GlobalDepositToken, Market, Orderbook, OrderStatus, Position, UserNonce};
use crate::program::constants::PROGRAM_ID;
use crate::program::error::{SdkError, SdkResult};
use crate::program::instructions::*;
use crate::program::orders::{derive_condition_id, SignedOrder};
use crate::program::pda::{
    get_all_conditional_mint_pdas, get_exchange_pda, get_global_deposit_token_pda,
    get_market_pda, get_order_status_pda, get_orderbook_pda, get_position_pda,
    get_user_global_deposit_pda, get_user_nonce_pda,
};
use crate::program::types::*;

/// Client for interacting with the Lightcone Pinocchio program.
pub struct LightconePinocchioClient {
    /// RPC client for Solana
    pub rpc_client: RpcClient,
    /// Program ID
    pub program_id: Pubkey,
}

impl LightconePinocchioClient {
    /// Create a new client with default program ID.
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_client: RpcClient::new_with_commitment(
                rpc_url.to_string(),
                CommitmentConfig::confirmed(),
            ),
            program_id: *PROGRAM_ID,
        }
    }

    /// Create a new client with custom program ID.
    pub fn with_program_id(rpc_url: &str, program_id: Pubkey) -> Self {
        Self {
            rpc_client: RpcClient::new_with_commitment(
                rpc_url.to_string(),
                CommitmentConfig::confirmed(),
            ),
            program_id,
        }
    }

    /// Create a new client with existing RpcClient.
    pub fn from_rpc_client(rpc_client: RpcClient) -> Self {
        Self {
            rpc_client,
            program_id: *PROGRAM_ID,
        }
    }

    // ========================================================================
    // Account Fetchers
    // ========================================================================

    /// Fetch the Exchange account.
    pub async fn get_exchange(&self) -> SdkResult<Exchange> {
        let (pda, _) = get_exchange_pda(&self.program_id);
        let account = self
            .rpc_client
            .get_account(&pda)
            .await
            .map_err(|e| SdkError::AccountNotFound(format!("Exchange: {}", e)))?;
        Exchange::deserialize(&account.data)
    }

    /// Fetch a Market account by ID.
    pub async fn get_market(&self, market_id: u64) -> SdkResult<Market> {
        let (pda, _) = get_market_pda(market_id, &self.program_id);
        self.get_market_by_pubkey(&pda).await
    }

    /// Fetch a Market account by pubkey.
    pub async fn get_market_by_pubkey(&self, market: &Pubkey) -> SdkResult<Market> {
        let account = self
            .rpc_client
            .get_account(market)
            .await
            .map_err(|e| SdkError::AccountNotFound(format!("Market: {}", e)))?;
        Market::deserialize(&account.data)
    }

    /// Fetch a Position account (returns None if not found).
    pub async fn get_position(
        &self,
        owner: &Pubkey,
        market: &Pubkey,
    ) -> SdkResult<Option<Position>> {
        let (pda, _) = get_position_pda(owner, market, &self.program_id);
        match self.rpc_client.get_account(&pda).await {
            Ok(account) => Ok(Some(Position::deserialize(&account.data)?)),
            Err(_) => Ok(None),
        }
    }

    /// Fetch an OrderStatus account (returns None if not found).
    pub async fn get_order_status(&self, order_hash: &[u8; 32]) -> SdkResult<Option<OrderStatus>> {
        let (pda, _) = get_order_status_pda(order_hash, &self.program_id);
        match self.rpc_client.get_account(&pda).await {
            Ok(account) => Ok(Some(OrderStatus::deserialize(&account.data)?)),
            Err(_) => Ok(None),
        }
    }

    /// Fetch a user's current nonce (returns 0 if not initialized).
    pub async fn get_user_nonce(&self, user: &Pubkey) -> SdkResult<u64> {
        let (pda, _) = get_user_nonce_pda(user, &self.program_id);
        match self.rpc_client.get_account(&pda).await {
            Ok(account) => {
                let user_nonce = UserNonce::deserialize(&account.data)?;
                Ok(user_nonce.nonce)
            }
            Err(_) => Ok(0),
        }
    }

    /// Get the next available nonce for a user as u32 (the current stored nonce value).
    ///
    /// Orders should be signed with this nonce value.
    /// Call `increment_nonce` to invalidate orders with the current nonce.
    ///
    /// Returns an error if the on-chain nonce exceeds u32::MAX.
    pub async fn get_next_nonce(&self, user: &Pubkey) -> SdkResult<u32> {
        let nonce = self.get_user_nonce(user).await?;
        u32::try_from(nonce).map_err(|_| SdkError::Overflow)
    }

    /// Get the next available market ID.
    pub async fn get_next_market_id(&self) -> SdkResult<u64> {
        let exchange = self.get_exchange().await?;
        Ok(exchange.market_count)
    }

    /// Fetch an Orderbook account by mint pair.
    pub async fn get_orderbook(
        &self,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
    ) -> SdkResult<Orderbook> {
        let (pda, _) = get_orderbook_pda(mint_a, mint_b, &self.program_id);
        let account = self
            .rpc_client
            .get_account(&pda)
            .await
            .map_err(|e| SdkError::AccountNotFound(format!("Orderbook: {}", e)))?;
        Orderbook::deserialize(&account.data)
    }

    /// Fetch a GlobalDepositToken account by mint.
    pub async fn get_global_deposit_token(
        &self,
        mint: &Pubkey,
    ) -> SdkResult<GlobalDepositToken> {
        let (pda, _) = get_global_deposit_token_pda(mint, &self.program_id);
        let account = self
            .rpc_client
            .get_account(&pda)
            .await
            .map_err(|e| SdkError::AccountNotFound(format!("GlobalDepositToken: {}", e)))?;
        GlobalDepositToken::deserialize(&account.data)
    }

    // ========================================================================
    // Transaction Builders
    // ========================================================================

    /// Get the latest blockhash for transaction building.
    pub async fn get_latest_blockhash(&self) -> SdkResult<Hash> {
        self.rpc_client
            .get_latest_blockhash()
            .await
            .map_err(SdkError::Rpc)
    }

    /// Build Initialize transaction.
    pub async fn initialize(&self, authority: &Pubkey) -> SdkResult<Transaction> {
        let ix = build_initialize_ix(authority, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(authority)))
    }

    /// Build CreateMarket transaction.
    pub async fn create_market(&self, params: CreateMarketParams) -> SdkResult<Transaction> {
        let market_id = self.get_next_market_id().await?;
        let ix = build_create_market_ix(&params, market_id, &self.program_id)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build AddDepositMint transaction.
    pub async fn add_deposit_mint(
        &self,
        params: AddDepositMintParams,
        market: &Pubkey,
        num_outcomes: u8,
    ) -> SdkResult<Transaction> {
        let ix = build_add_deposit_mint_ix(&params, market, num_outcomes, &self.program_id)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.payer)))
    }

    /// Build MintCompleteSet transaction.
    pub async fn mint_complete_set(
        &self,
        params: MintCompleteSetParams,
        num_outcomes: u8,
    ) -> SdkResult<Transaction> {
        let ix = build_mint_complete_set_ix(&params, num_outcomes, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build MergeCompleteSet transaction.
    pub async fn merge_complete_set(
        &self,
        params: MergeCompleteSetParams,
        num_outcomes: u8,
    ) -> SdkResult<Transaction> {
        let ix = build_merge_complete_set_ix(&params, num_outcomes, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build CancelOrder transaction.
    pub async fn cancel_order(
        &self,
        maker: &Pubkey,
        market: &Pubkey,
        order: &SignedOrder,
    ) -> SdkResult<Transaction> {
        let ix = build_cancel_order_ix(maker, market, order, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(maker)))
    }

    /// Build IncrementNonce transaction.
    pub async fn increment_nonce(&self, user: &Pubkey) -> SdkResult<Transaction> {
        let ix = build_increment_nonce_ix(user, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(user)))
    }

    /// Build SettleMarket transaction.
    pub async fn settle_market(&self, params: SettleMarketParams) -> SdkResult<Transaction> {
        let ix = build_settle_market_ix(&params, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.oracle)))
    }

    /// Build RedeemWinnings transaction.
    pub async fn redeem_winnings(
        &self,
        params: RedeemWinningsParams,
        winning_outcome: u8,
    ) -> SdkResult<Transaction> {
        let ix = build_redeem_winnings_ix(&params, winning_outcome, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build SetPaused transaction.
    pub async fn set_paused(&self, authority: &Pubkey, paused: bool) -> SdkResult<Transaction> {
        let ix = build_set_paused_ix(authority, paused, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(authority)))
    }

    /// Build SetOperator transaction.
    pub async fn set_operator(
        &self,
        authority: &Pubkey,
        new_operator: &Pubkey,
    ) -> SdkResult<Transaction> {
        let ix = build_set_operator_ix(authority, new_operator, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(authority)))
    }

    /// Build WithdrawFromPosition transaction.
    pub async fn withdraw_from_position(
        &self,
        params: WithdrawFromPositionParams,
        is_token_2022: bool,
    ) -> SdkResult<Transaction> {
        let ix = build_withdraw_from_position_ix(&params, is_token_2022, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build ActivateMarket transaction.
    pub async fn activate_market(&self, params: ActivateMarketParams) -> SdkResult<Transaction> {
        let ix = build_activate_market_ix(&params, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build MatchOrdersMulti transaction.
    pub async fn match_orders_multi(
        &self,
        params: MatchOrdersMultiParams,
    ) -> SdkResult<Transaction> {
        let ix = build_match_orders_multi_ix(&params, &self.program_id)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }

    /// Build CreateOrderbook transaction.
    pub async fn create_orderbook(
        &self,
        params: CreateOrderbookParams,
    ) -> SdkResult<Transaction> {
        let ix = build_create_orderbook_ix(&params, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.payer)))
    }

    /// Build SetAuthority transaction.
    pub async fn set_authority(&self, params: SetAuthorityParams) -> SdkResult<Transaction> {
        let ix = build_set_authority_ix(&params, &self.program_id);
        Ok(Transaction::new_with_payer(
            &[ix],
            Some(&params.current_authority),
        ))
    }

    /// Build WhitelistDepositToken transaction.
    pub async fn whitelist_deposit_token(
        &self,
        params: WhitelistDepositTokenParams,
    ) -> SdkResult<Transaction> {
        let ix = build_whitelist_deposit_token_ix(&params, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build DepositToGlobal transaction.
    pub async fn deposit_to_global(
        &self,
        params: DepositToGlobalParams,
    ) -> SdkResult<Transaction> {
        let ix = build_deposit_to_global_ix(&params, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build GlobalToMarketDeposit transaction.
    pub async fn global_to_market_deposit(
        &self,
        params: GlobalToMarketDepositParams,
        num_outcomes: u8,
    ) -> SdkResult<Transaction> {
        let ix = build_global_to_market_deposit_ix(&params, num_outcomes, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build InitPositionTokens transaction.
    pub async fn init_position_tokens(
        &self,
        params: InitPositionTokensParams,
        num_outcomes: u8,
    ) -> SdkResult<Transaction> {
        let ix = build_init_position_tokens_ix(&params, num_outcomes, &self.program_id);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build DepositAndSwap transaction.
    pub async fn deposit_and_swap(
        &self,
        params: DepositAndSwapParams,
        num_outcomes: u8,
    ) -> SdkResult<Transaction> {
        let ix = build_deposit_and_swap_ix(&params, num_outcomes, &self.program_id)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }

    // ========================================================================
    // Order Helpers
    // ========================================================================

    /// Create an unsigned bid order.
    pub fn create_bid_order(&self, params: BidOrderParams) -> SignedOrder {
        SignedOrder::new_bid(params)
    }

    /// Create an unsigned ask order.
    pub fn create_ask_order(&self, params: AskOrderParams) -> SignedOrder {
        SignedOrder::new_ask(params)
    }

    /// Create and sign a bid order.
    #[cfg(feature = "native-client")]
    pub fn create_signed_bid_order(
        &self,
        params: BidOrderParams,
        keypair: &Keypair,
    ) -> SignedOrder {
        SignedOrder::new_bid_signed(params, keypair)
    }

    /// Create and sign an ask order.
    #[cfg(feature = "native-client")]
    pub fn create_signed_ask_order(
        &self,
        params: AskOrderParams,
        keypair: &Keypair,
    ) -> SignedOrder {
        SignedOrder::new_ask_signed(params, keypair)
    }

    /// Compute the hash of an order.
    pub fn hash_order(&self, order: &SignedOrder) -> [u8; 32] {
        order.hash()
    }

    /// Sign an order with the given keypair.
    #[cfg(feature = "native-client")]
    pub fn sign_order(&self, order: &mut SignedOrder, keypair: &Keypair) {
        order.sign(keypair);
    }

    // ========================================================================
    // Utility Functions
    // ========================================================================

    /// Derive the condition ID for a market.
    pub fn derive_condition_id(
        &self,
        oracle: &Pubkey,
        question_id: &[u8; 32],
        num_outcomes: u8,
    ) -> [u8; 32] {
        derive_condition_id(oracle, question_id, num_outcomes)
    }

    /// Get all conditional mint pubkeys for a market.
    pub fn get_conditional_mints(
        &self,
        market: &Pubkey,
        deposit_mint: &Pubkey,
        num_outcomes: u8,
    ) -> Vec<Pubkey> {
        get_all_conditional_mint_pdas(market, deposit_mint, num_outcomes, &self.program_id)
            .into_iter()
            .map(|(pubkey, _)| pubkey)
            .collect()
    }

    /// Get the Exchange PDA.
    pub fn get_exchange_pda(&self) -> Pubkey {
        get_exchange_pda(&self.program_id).0
    }

    /// Get a Market PDA.
    pub fn get_market_pda(&self, market_id: u64) -> Pubkey {
        get_market_pda(market_id, &self.program_id).0
    }

    /// Get a Position PDA.
    pub fn get_position_pda(&self, owner: &Pubkey, market: &Pubkey) -> Pubkey {
        get_position_pda(owner, market, &self.program_id).0
    }

    /// Get an Order Status PDA.
    pub fn get_order_status_pda(&self, order_hash: &[u8; 32]) -> Pubkey {
        get_order_status_pda(order_hash, &self.program_id).0
    }

    /// Get a User Nonce PDA.
    pub fn get_user_nonce_pda(&self, user: &Pubkey) -> Pubkey {
        get_user_nonce_pda(user, &self.program_id).0
    }

    /// Get an Orderbook PDA.
    pub fn get_orderbook_pda(&self, mint_a: &Pubkey, mint_b: &Pubkey) -> Pubkey {
        get_orderbook_pda(mint_a, mint_b, &self.program_id).0
    }

    /// Get a GlobalDepositToken PDA.
    pub fn get_global_deposit_token_pda(&self, mint: &Pubkey) -> Pubkey {
        get_global_deposit_token_pda(mint, &self.program_id).0
    }

    /// Get a User Global Deposit PDA.
    pub fn get_user_global_deposit_pda(&self, user: &Pubkey, mint: &Pubkey) -> Pubkey {
        get_user_global_deposit_pda(user, mint, &self.program_id).0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LightconePinocchioClient::new("https://api.devnet.solana.com");
        assert_eq!(client.program_id, *PROGRAM_ID);
    }

    #[test]
    fn test_client_with_custom_program_id() {
        let custom_id = Pubkey::new_unique();
        let client =
            LightconePinocchioClient::with_program_id("https://api.devnet.solana.com", custom_id);
        assert_eq!(client.program_id, custom_id);
    }

    #[test]
    fn test_pda_helpers() {
        let client = LightconePinocchioClient::new("https://api.devnet.solana.com");

        let exchange_pda = client.get_exchange_pda();
        assert_ne!(exchange_pda, Pubkey::default());

        let market_pda = client.get_market_pda(0);
        assert_ne!(market_pda, Pubkey::default());

        let owner = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let position_pda = client.get_position_pda(&owner, &market);
        assert_ne!(position_pda, Pubkey::default());

        let mint_a = Pubkey::new_unique();
        let mint_b = Pubkey::new_unique();
        let orderbook_pda = client.get_orderbook_pda(&mint_a, &mint_b);
        assert_ne!(orderbook_pda, Pubkey::default());
    }

    #[test]
    fn test_create_bid_order() {
        let client = LightconePinocchioClient::new("https://api.devnet.solana.com");

        let params = BidOrderParams {
            nonce: 1,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            amount_in: 1000,
            amount_out: 500,
            expiration: 0,
        };

        let order = client.create_bid_order(params.clone());
        assert_eq!(order.nonce, params.nonce);
        assert_eq!(order.maker, params.maker);
        assert_eq!(order.amount_in, params.amount_in);
    }

    #[test]
    fn test_condition_id_derivation() {
        let client = LightconePinocchioClient::new("https://api.devnet.solana.com");

        let oracle = Pubkey::new_unique();
        let question_id = [1u8; 32];
        let num_outcomes = 3;

        let condition_id1 = client.derive_condition_id(&oracle, &question_id, num_outcomes);
        let condition_id2 = client.derive_condition_id(&oracle, &question_id, num_outcomes);

        assert_eq!(condition_id1, condition_id2);
    }

    #[test]
    fn test_get_conditional_mints() {
        let client = LightconePinocchioClient::new("https://api.devnet.solana.com");

        let market = Pubkey::new_unique();
        let deposit_mint = Pubkey::new_unique();

        let mints = client.get_conditional_mints(&market, &deposit_mint, 3);
        assert_eq!(mints.len(), 3);

        // All mints should be unique
        assert_ne!(mints[0], mints[1]);
        assert_ne!(mints[1], mints[2]);
        assert_ne!(mints[0], mints[2]);
    }
}
