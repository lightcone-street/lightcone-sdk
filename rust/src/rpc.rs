//! RPC sub-client — exchange-level on-chain fetchers, global deposit helpers, and blockhash access.

use crate::client::LightconeClient;
use crate::error::SdkError;
use solana_pubkey::Pubkey;

#[cfg(feature = "solana-rpc")]
use solana_client::nonblocking::rpc_client::RpcClient as SolanaRpcClient;

/// Resolve the Solana RPC client from a `LightconeClient`, or error if not configured.
#[cfg(feature = "solana-rpc")]
pub(crate) fn require_solana_rpc(client: &LightconeClient) -> Result<&SolanaRpcClient, SdkError> {
    client.solana_rpc_client.as_ref().ok_or_else(|| {
        SdkError::Other("RPC client not configured — use .rpc_url() on the builder".to_string())
    })
}

pub struct Rpc<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Rpc<'a> {
    /// Get the underlying Solana `RpcClient`, returning an error if not configured.
    ///
    /// Prefer the typed methods on each sub-client for common operations.
    /// Use this when you need the raw client (e.g. `send_and_confirm_transaction`).
    #[cfg(feature = "solana-rpc")]
    pub fn inner(&self) -> Result<&SolanaRpcClient, SdkError> {
        require_solana_rpc(self.client)
    }

    // ── PDA helpers ──────────────────────────────────────────────────────

    /// Get the Exchange PDA.
    pub fn get_exchange_pda(&self) -> Pubkey {
        crate::program::pda::get_exchange_pda(&self.client.program_id).0
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
        let rpc = require_solana_rpc(self.client)?;
        rpc.get_latest_blockhash()
            .await
            .map_err(|e| SdkError::Program(crate::program::error::SdkError::Rpc(e)))
    }

    /// Fetch the Exchange account.
    pub async fn get_exchange(&self) -> Result<crate::program::accounts::Exchange, SdkError> {
        let rpc = require_solana_rpc(self.client)?;
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

    /// Fetch a GlobalDepositToken account by mint.
    pub async fn get_global_deposit_token(
        &self,
        mint: &Pubkey,
    ) -> Result<crate::program::accounts::GlobalDepositToken, SdkError> {
        let rpc = require_solana_rpc(self.client)?;
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
