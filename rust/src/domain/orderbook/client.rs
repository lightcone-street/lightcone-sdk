//! Orderbooks sub-client — depth and on-chain orderbook operations.

use crate::client::LightconeClient;
use crate::domain::orderbook::wire::OrderbookDepthResponse;
use crate::error::SdkError;
use crate::http::RetryPolicy;
use solana_pubkey::Pubkey;

pub struct Orderbooks<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Orderbooks<'a> {
    // ── PDA helpers ──────────────────────────────────────────────────────

    /// Get the Orderbook PDA.
    pub fn pda(&self, mint_a: &Pubkey, mint_b: &Pubkey) -> Pubkey {
        crate::program::pda::get_orderbook_pda(mint_a, mint_b, &self.client.program_id).0
    }

    // ── HTTP methods ─────────────────────────────────────────────────────

    /// Get live orderbook depth (always fresh).
    pub async fn get(
        &self,
        orderbook_id: &str,
        depth: Option<u32>,
    ) -> Result<OrderbookDepthResponse, SdkError> {
        let mut url = format!("{}/api/orderbook/{}", self.client.http.base_url(), orderbook_id);
        if let Some(d) = depth {
            url = format!("{}?depth={}", url, d);
        }
        Ok(self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// On-chain account fetchers (require RPC)
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "solana-rpc")]
impl<'a> Orderbooks<'a> {
    /// Fetch an Orderbook account by mint pair.
    pub async fn get_onchain(
        &self,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
    ) -> Result<crate::program::accounts::Orderbook, SdkError> {
        let rpc = crate::rpc::require_solana_rpc(self.client)?;
        let pda = self.pda(mint_a, mint_b);
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
}
