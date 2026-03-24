//! Markets sub-client — fetch, search, and on-chain market operations.

use crate::client::LightconeClient;
use crate::domain::market::wire::{MarketSearchResult, MarketsResponse, SingleMarketResponse};
use crate::domain::market::{self, Market, Status};
use crate::error::SdkError;
use crate::http::RetryPolicy;
use serde::{Deserialize, Serialize};
use solana_pubkey::Pubkey;

/// Result of fetching multiple markets. Contains valid markets and any
/// validation errors encountered (invalid markets are skipped, not fatal).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsResult {
    pub markets: Vec<Market>,
    pub validation_errors: Vec<String>,
}

pub struct Markets<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Markets<'a> {
    /// Fetch markets (cursor-based pagination).
    ///
    /// Only returns Active and Resolved markets. Markets that fail validation
    /// are skipped and their errors are returned in `MarketsResult::validation_errors`.
    pub async fn get(
        &self,
        cursor: Option<i64>,
        limit: Option<u32>,
    ) -> Result<MarketsResult, SdkError> {
        let base = self.client.http.base_url();
        let mut url = format!("{}/api/markets", base);
        let mut params = Vec::new();
        if let Some(c) = cursor {
            params.push(format!("cursor={}", c));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let resp: MarketsResponse = self.client.http.get(&url, RetryPolicy::Idempotent).await?;

        let mut markets = Vec::new();
        let mut validation_errors = Vec::new();

        for mr in resp.markets {
            match Market::try_from(mr) {
                Ok(market) => {
                    if matches!(market.status, Status::Active | Status::Resolved) {
                        markets.push(market);
                    } else {
                        tracing::debug!(
                            "Skipped market {} (status: {})",
                            market.pubkey,
                            market.status.as_str()
                        );
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    tracing::warn!("Market validation error: {}", msg);
                    validation_errors.push(msg);
                }
            }
        }

        Ok(MarketsResult {
            markets,
            validation_errors,
        })
    }

    /// Fetch a market by slug.
    pub async fn get_by_slug(&self, slug: &str) -> Result<Market, SdkError> {
        let url = format!(
            "{}/api/markets/by-slug/{}",
            self.client.http.base_url(),
            slug
        );
        let resp: SingleMarketResponse =
            self.client.http.get(&url, RetryPolicy::Idempotent).await?;

        resp.market
            .try_into()
            .map_err(|e: market::ValidationError| SdkError::Validation(e.to_string()))
    }

    /// Fetch a market by on-chain pubkey.
    pub async fn get_by_pubkey(&self, pubkey: &str) -> Result<Market, SdkError> {
        let url = format!("{}/api/markets/{}", self.client.http.base_url(), pubkey);
        let resp: SingleMarketResponse =
            self.client.http.get(&url, RetryPolicy::Idempotent).await?;

        resp.market
            .try_into()
            .map_err(|e: market::ValidationError| SdkError::Validation(e.to_string()))
    }

    /// Search markets by query string.
    pub async fn search(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<MarketSearchResult>, SdkError> {
        let encoded = urlencoding::encode(query);
        let mut url = format!(
            "{}/api/markets/search/by-query/{}",
            self.client.http.base_url(),
            encoded
        );
        if let Some(l) = limit {
            url = format!("{}?limit={}", url, l);
        }
        Ok(self.client.http.get(&url, RetryPolicy::Idempotent).await?)
    }

    /// Get featured markets. Only returns Active and Resolved markets.
    pub async fn featured(&self) -> Result<Vec<MarketSearchResult>, SdkError> {
        let url = format!(
            "{}/api/markets/search/featured",
            self.client.http.base_url()
        );
        let results: Vec<MarketSearchResult> =
            self.client.http.get(&url, RetryPolicy::Idempotent).await?;

        let (kept, skipped): (Vec<_>, Vec<_>) = results
            .into_iter()
            .partition(|r| matches!(r.market_status, Status::Active | Status::Resolved));

        for r in &skipped {
            tracing::debug!(
                "Skipped featured market '{}' (status: {})",
                r.slug,
                r.market_status.as_str()
            );
        }

        Ok(kept)
    }

    // ── PDA helpers ──────────────────────────────────────────────────────

    /// Get the Market PDA for a given market ID.
    pub fn pda(&self, market_id: u64) -> Pubkey {
        crate::program::pda::get_market_pda(market_id, &self.client.program_id).0
    }

    // ── Market helpers ───────────────────────────────────────────────────

    /// Derive the condition ID for a market.
    pub fn derive_condition_id(
        &self,
        oracle: &Pubkey,
        question_id: &[u8; 32],
        num_outcomes: u8,
    ) -> [u8; 32] {
        crate::program::orders::derive_condition_id(oracle, question_id, num_outcomes)
    }

    /// Get all conditional mint pubkeys for a market.
    pub fn get_conditional_mints(
        &self,
        market: &Pubkey,
        deposit_mint: &Pubkey,
        num_outcomes: u8,
    ) -> Vec<Pubkey> {
        crate::program::pda::get_all_conditional_mint_pdas(
            market,
            deposit_mint,
            num_outcomes,
            &self.client.program_id,
        )
        .into_iter()
        .map(|(pubkey, _)| pubkey)
        .collect()
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// On-chain account fetchers (require RPC)
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "solana-rpc")]
impl<'a> Markets<'a> {
    /// Fetch a Market account by on-chain pubkey.
    pub async fn get_onchain(
        &self,
        market: &Pubkey,
    ) -> Result<crate::program::accounts::Market, SdkError> {
        let rpc = crate::rpc::require_solana_rpc(self.client)?;
        let account = rpc.get_account(market).await.map_err(|e| {
            SdkError::Program(crate::program::error::SdkError::AccountNotFound(format!(
                "Market: {}",
                e
            )))
        })?;
        Ok(crate::program::accounts::Market::deserialize(
            &account.data,
        )?)
    }

    /// Fetch a Market account by ID.
    pub async fn get_by_id_onchain(
        &self,
        market_id: u64,
    ) -> Result<crate::program::accounts::Market, SdkError> {
        let pda = self.pda(market_id);
        self.get_onchain(&pda).await
    }

    /// Get the next available market ID.
    pub async fn next_id(&self) -> Result<u64, SdkError> {
        let exchange = self.client.rpc().get_exchange().await?;
        Ok(exchange.market_count)
    }
}
