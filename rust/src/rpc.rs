//! RPC sub-client — on-chain account fetchers, PDA helpers, and blockhash access.

use crate::client::LightconeClient;
use crate::error::SdkError;
use solana_pubkey::Pubkey;

#[cfg(feature = "solana-rpc")]
use solana_client::nonblocking::rpc_client::RpcClient as SolanaRpcClient;

pub struct Rpc<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Rpc<'a> {
    /// Get the underlying Solana `RpcClient`, returning an error if not configured.
    #[cfg(feature = "solana-rpc")]
    pub fn inner(&self) -> Result<&SolanaRpcClient, SdkError> {
        self.client
            .rpc_client
            .as_ref()
            .ok_or_else(|| {
                SdkError::Other(
                    "RPC client not configured — use .rpc_url() on the builder".to_string(),
                )
            })
    }

    // ── PDA helpers ──────────────────────────────────────────────────────

    /// Get the Exchange PDA.
    pub fn get_exchange_pda(&self) -> Pubkey {
        crate::program::pda::get_exchange_pda(&self.client.program_id).0
    }

    /// Get a Market PDA.
    pub fn get_market_pda(&self, market_id: u64) -> Pubkey {
        crate::program::pda::get_market_pda(market_id, &self.client.program_id).0
    }

    /// Get a Position PDA.
    pub fn get_position_pda(&self, owner: &Pubkey, market: &Pubkey) -> Pubkey {
        crate::program::pda::get_position_pda(owner, market, &self.client.program_id).0
    }

    /// Get an Order Status PDA.
    pub fn get_order_status_pda(&self, order_hash: &[u8; 32]) -> Pubkey {
        crate::program::pda::get_order_status_pda(order_hash, &self.client.program_id).0
    }

    /// Get a User Nonce PDA.
    pub fn get_user_nonce_pda(&self, user: &Pubkey) -> Pubkey {
        crate::program::pda::get_user_nonce_pda(user, &self.client.program_id).0
    }

    /// Get an Orderbook PDA.
    pub fn get_orderbook_pda(&self, mint_a: &Pubkey, mint_b: &Pubkey) -> Pubkey {
        crate::program::pda::get_orderbook_pda(mint_a, mint_b, &self.client.program_id).0
    }

    /// Get a GlobalDepositToken PDA.
    pub fn get_global_deposit_token_pda(&self, mint: &Pubkey) -> Pubkey {
        crate::program::pda::get_global_deposit_token_pda(mint, &self.client.program_id).0
    }

    /// Get a User Global Deposit PDA.
    pub fn get_user_global_deposit_pda(&self, user: &Pubkey, mint: &Pubkey) -> Pubkey {
        crate::program::pda::get_user_global_deposit_pda(user, mint, &self.client.program_id).0
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// On-chain account fetchers (require RPC)
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "solana-rpc")]
impl<'a> Rpc<'a> {
    /// Get the latest blockhash for transaction building.
    pub async fn get_latest_blockhash(&self) -> Result<solana_hash::Hash, SdkError> {
        let rpc = self.inner()?;
        rpc.get_latest_blockhash()
            .await
            .map_err(|e| SdkError::Program(crate::program::error::SdkError::Rpc(e)))
    }

    /// Fetch the Exchange account.
    pub async fn get_exchange(&self) -> Result<crate::program::accounts::Exchange, SdkError> {
        let rpc = self.inner()?;
        let (pda, _) = crate::program::pda::get_exchange_pda(&self.client.program_id);
        let account = rpc
            .get_account(&pda)
            .await
            .map_err(|e| {
                SdkError::Program(crate::program::error::SdkError::AccountNotFound(format!(
                    "Exchange: {}",
                    e
                )))
            })?;
        Ok(crate::program::accounts::Exchange::deserialize(&account.data)?)
    }

    /// Fetch a Market account by ID.
    pub async fn get_market_by_id(
        &self,
        market_id: u64,
    ) -> Result<crate::program::accounts::Market, SdkError> {
        let (pda, _) = crate::program::pda::get_market_pda(market_id, &self.client.program_id);
        self.get_market_onchain(&pda).await
    }

    /// Fetch a Market account by pubkey.
    pub async fn get_market_onchain(
        &self,
        market: &Pubkey,
    ) -> Result<crate::program::accounts::Market, SdkError> {
        let rpc = self.inner()?;
        let account = rpc
            .get_account(market)
            .await
            .map_err(|e| {
                SdkError::Program(crate::program::error::SdkError::AccountNotFound(format!(
                    "Market: {}",
                    e
                )))
            })?;
        Ok(crate::program::accounts::Market::deserialize(&account.data)?)
    }

    /// Fetch a Position account (returns None if not found).
    pub async fn get_position_onchain(
        &self,
        owner: &Pubkey,
        market: &Pubkey,
    ) -> Result<Option<crate::program::accounts::Position>, SdkError> {
        let rpc = self.inner()?;
        let (pda, _) =
            crate::program::pda::get_position_pda(owner, market, &self.client.program_id);
        match rpc.get_account(&pda).await {
            Ok(account) => Ok(Some(
                crate::program::accounts::Position::deserialize(&account.data)?,
            )),
            Err(_) => Ok(None),
        }
    }

    /// Fetch an OrderStatus account (returns None if not found).
    pub async fn get_order_status(
        &self,
        order_hash: &[u8; 32],
    ) -> Result<Option<crate::program::accounts::OrderStatus>, SdkError> {
        let rpc = self.inner()?;
        let (pda, _) =
            crate::program::pda::get_order_status_pda(order_hash, &self.client.program_id);
        match rpc.get_account(&pda).await {
            Ok(account) => Ok(Some(
                crate::program::accounts::OrderStatus::deserialize(&account.data)?,
            )),
            Err(_) => Ok(None),
        }
    }

    /// Fetch a user's current nonce (returns 0 if not initialized).
    pub async fn get_user_nonce(&self, user: &Pubkey) -> Result<u64, SdkError> {
        let rpc = self.inner()?;
        let (pda, _) = crate::program::pda::get_user_nonce_pda(user, &self.client.program_id);
        match rpc.get_account(&pda).await {
            Ok(account) => {
                let user_nonce =
                    crate::program::accounts::UserNonce::deserialize(&account.data)?;
                Ok(user_nonce.nonce)
            }
            Err(_) => Ok(0),
        }
    }

    /// Get the current on-chain nonce for a user as u32.
    pub async fn get_current_nonce(&self, user: &Pubkey) -> Result<u32, SdkError> {
        let nonce = self.get_user_nonce(user).await?;
        u32::try_from(nonce)
            .map_err(|_| SdkError::Program(crate::program::error::SdkError::Overflow))
    }

    /// Get the next available market ID.
    pub async fn get_next_market_id(&self) -> Result<u64, SdkError> {
        let exchange = self.get_exchange().await?;
        Ok(exchange.market_count)
    }

    /// Fetch an Orderbook account by mint pair.
    pub async fn get_orderbook_onchain(
        &self,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
    ) -> Result<crate::program::accounts::Orderbook, SdkError> {
        let rpc = self.inner()?;
        let (pda, _) =
            crate::program::pda::get_orderbook_pda(mint_a, mint_b, &self.client.program_id);
        let account = rpc
            .get_account(&pda)
            .await
            .map_err(|e| {
                SdkError::Program(crate::program::error::SdkError::AccountNotFound(format!(
                    "Orderbook: {}",
                    e
                )))
            })?;
        Ok(crate::program::accounts::Orderbook::deserialize(&account.data)?)
    }

    /// Fetch a GlobalDepositToken account by mint.
    pub async fn get_global_deposit_token(
        &self,
        mint: &Pubkey,
    ) -> Result<crate::program::accounts::GlobalDepositToken, SdkError> {
        let rpc = self.inner()?;
        let (pda, _) =
            crate::program::pda::get_global_deposit_token_pda(mint, &self.client.program_id);
        let account = rpc
            .get_account(&pda)
            .await
            .map_err(|e| {
                SdkError::Program(crate::program::error::SdkError::AccountNotFound(format!(
                    "GlobalDepositToken: {}",
                    e
                )))
            })?;
        Ok(crate::program::accounts::GlobalDepositToken::deserialize(
            &account.data,
        )?)
    }
}
