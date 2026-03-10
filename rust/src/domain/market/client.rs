//! Markets sub-client — fetch, search, and on-chain market operations.

use crate::client::LightconeClient;
use crate::domain::market::{self, Market, Status};
use crate::domain::market::wire::{MarketSearchResult, MarketsResponse, SingleMarketResponse};
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::instructions;
use crate::program::types::{MergeCompleteSetParams, MintCompleteSetParams};
use serde::{Deserialize, Serialize};
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;

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

        let resp: MarketsResponse = self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?;

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
        let url = format!("{}/api/markets/by-slug/{}", self.client.http.base_url(), slug);
        let resp: SingleMarketResponse = self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?;

        resp.market
            .try_into()
            .map_err(|e: market::ValidationError| SdkError::Validation(e.to_string()))
    }

    /// Fetch a market by on-chain pubkey.
    pub async fn get_by_pubkey(&self, pubkey: &str) -> Result<Market, SdkError> {
        let url = format!("{}/api/markets/{}", self.client.http.base_url(), pubkey);
        let resp: SingleMarketResponse = self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?;

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
        Ok(self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?)
    }

    /// Get featured markets. Only returns Active and Resolved markets.
    pub async fn featured(&self) -> Result<Vec<MarketSearchResult>, SdkError> {
        let url = format!("{}/api/markets/search/featured", self.client.http.base_url());
        let results: Vec<MarketSearchResult> = self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?;

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

    // ── On-chain transaction builders ────────────────────────────────────

    /// Build MintCompleteSet transaction.
    pub fn mint_complete_set_ix(
        &self,
        params: MintCompleteSetParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_mint_complete_set_ix(&params, num_outcomes, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build MergeCompleteSet transaction.
    pub fn merge_complete_set_ix(
        &self,
        params: MergeCompleteSetParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_merge_complete_set_ix(&params, num_outcomes, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
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
