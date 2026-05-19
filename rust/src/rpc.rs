//! RPC sub-client — exchange-level on-chain fetchers, global deposit helpers, and blockhash access.

use crate::client::LightconeClient;
#[cfg(feature = "solana-rpc")]
use crate::error::SdkError;
use solana_pubkey::Pubkey;

#[cfg(feature = "solana-rpc")]
use solana_client::nonblocking::rpc_client::RpcClient as SolanaRpcClient;

#[cfg(feature = "solana-rpc")]
use crate::rpc_failover::{is_infrastructure_error_solana, with_failover, ActiveRpc};

#[cfg(feature = "solana-rpc")]
use std::pin::Pin;

/// Resolve the currently-active Solana RPC client based on failover state.
#[cfg(feature = "solana-rpc")]
pub(crate) async fn resolve_solana_rpc(
    client: &LightconeClient,
) -> Result<&SolanaRpcClient, SdkError> {
    let state = client.rpc_failover_state.read().await;
    let active = state.active();
    drop(state);
    match active {
        ActiveRpc::Primary => client.primary_solana_rpc_client.as_ref(),
        ActiveRpc::Backup => client
            .backup_solana_rpc_client
            .as_ref()
            .or(client.primary_solana_rpc_client.as_ref()),
    }
    .ok_or_else(|| {
        SdkError::Other("RPC client not configured — use .rpc_url() on the builder".to_string())
    })
}

/// Execute a native RPC operation with fast retry + failover.
#[cfg(feature = "solana-rpc")]
async fn solana_rpc_with_failover<F, T>(
    client: &LightconeClient,
    operation: F,
) -> Result<T, solana_client::client_error::ClientError>
where
    for<'r> F: Fn(
        &'r SolanaRpcClient,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<T, solana_client::client_error::ClientError>>
                + Send
                + 'r,
        >,
    >,
{
    let primary = client.primary_solana_rpc_client.as_ref();
    let backup = client.backup_solana_rpc_client.as_ref();

    with_failover(
        &client.rpc_failover_state,
        |target| {
            let rpc = match target {
                ActiveRpc::Primary => primary.or(backup),
                ActiveRpc::Backup => backup.or(primary),
            }
            .expect("RPC client not configured");
            operation(rpc)
        },
        backup.is_some(),
        is_infrastructure_error_solana,
    )
    .await
}

pub struct Rpc<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Rpc<'a> {
    /// Get the currently-active underlying Solana `RpcClient`.
    ///
    /// Prefer the typed methods (`get_exchange`, etc.) for common operations —
    /// they include automatic failover. Direct use of `inner()` bypasses the
    /// retry/failover wrapper.
    #[cfg(feature = "solana-rpc")]
    pub async fn inner(&self) -> Result<&SolanaRpcClient, SdkError> {
        resolve_solana_rpc(self.client).await
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
        solana_rpc_with_failover(self.client, |rpc| {
            Box::pin(async move { rpc.get_latest_blockhash().await })
        })
        .await
        .map_err(|error| SdkError::Program(crate::program::error::SdkError::Rpc(error)))
    }

    /// Fetch the Exchange account.
    pub async fn get_exchange(&self) -> Result<crate::program::accounts::Exchange, SdkError> {
        let (pda, _) = crate::program::pda::get_exchange_pda(&self.client.program_id);
        let account = solana_rpc_with_failover(self.client, move |rpc| {
            Box::pin(async move { rpc.get_account(&pda).await })
        })
        .await
        .map_err(|error| {
            SdkError::Program(crate::program::error::SdkError::AccountNotFound(format!(
                "Exchange: {}",
                error
            )))
        })?;
        Ok(crate::program::accounts::Exchange::deserialize(
            &account.data,
        )?)
    }

    /// Fetch a GlobalDepositToken account by mint.
    pub async fn get_global_deposit_token(
        &self,
        mint: &Pubkey,
    ) -> Result<crate::program::accounts::GlobalDepositToken, SdkError> {
        let (pda, _) =
            crate::program::pda::get_global_deposit_token_pda(mint, &self.client.program_id);
        let account = solana_rpc_with_failover(self.client, move |rpc| {
            Box::pin(async move { rpc.get_account(&pda).await })
        })
        .await
        .map_err(|error| {
            SdkError::Program(crate::program::error::SdkError::AccountNotFound(format!(
                "GlobalDepositToken: {}",
                error
            )))
        })?;
        Ok(crate::program::accounts::GlobalDepositToken::deserialize(
            &account.data,
        )?)
    }
}
