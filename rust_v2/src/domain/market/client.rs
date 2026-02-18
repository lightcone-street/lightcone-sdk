//! Markets sub-client — fetch, search.

use crate::client::LightconeClient;
use crate::domain::market::{self, Market};
use crate::domain::market::wire::{MarketResponse, MarketSearchResult, MarketsResponse};
use crate::error::SdkError;
use crate::http::RetryPolicy;

pub struct Markets<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Markets<'a> {
    /// Fetch all markets (paginated).
    pub async fn get(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<Market>, SdkError> {
        let base = self.client.http.base_url();
        let mut url = format!("{}/api/markets", base);
        let mut params = Vec::new();
        if let Some(p) = page {
            params.push(format!("page={}", p));
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

        resp.markets
            .into_iter()
            .map(|mr| {
                mr.try_into()
                    .map_err(|e: market::ValidationError| SdkError::Validation(e.to_string()))
            })
            .collect()
    }

    /// Fetch a market by slug.
    pub async fn get_by_slug(&self, slug: &str) -> Result<Market, SdkError> {
        let url = format!("{}/api/markets/by-slug/{}", self.client.http.base_url(), slug);
        let resp: MarketResponse = self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?;

        resp.try_into()
            .map_err(|e: market::ValidationError| SdkError::Validation(e.to_string()))
    }

    /// Search markets.
    pub async fn search(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<MarketSearchResult>, SdkError> {
        let mut url = format!(
            "{}/api/markets/search?q={}",
            self.client.http.base_url(),
            urlencoding::encode(query)
        );
        if let Some(l) = limit {
            url = format!("{}&limit={}", url, l);
        }
        Ok(self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?)
    }

    /// Get featured markets.
    pub async fn featured(&self) -> Result<Vec<MarketSearchResult>, SdkError> {
        let url = format!("{}/api/markets/featured", self.client.http.base_url());
        Ok(self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?)
    }
}
