//! Markets sub-client — fetch, search, cache.

use crate::client::LightconeClient;
use crate::domain::market::{self, Market};
use crate::domain::market::wire::MarketSearchResult;
use crate::error::{HttpError, SdkError};
use std::time::Instant;

/// Sub-client for market operations.
pub struct Markets<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Markets<'a> {
    /// Get a market by pubkey. Uses TTL cache.
    pub async fn get(&self, pubkey: &str) -> Result<Market, SdkError> {
        {
            let cache = self.client.market_cache.read().await;
            if let Some((market, fetched_at)) = cache.get(pubkey) {
                if fetched_at.elapsed() < self.client.market_cache_ttl {
                    return Ok(market.clone());
                }
            }
        }

        let resp = self.client.http.get_markets(None, None).await?;
        for mr in resp.markets {
            if mr.market_pubkey == pubkey {
                let market: Market = mr.try_into().map_err(|e: market::ValidationError| {
                    SdkError::Validation(e.to_string())
                })?;
                self.cache_market(&market).await;
                return Ok(market);
            }
        }

        Err(SdkError::Http(HttpError::NotFound(format!(
            "Market not found: {}",
            pubkey
        ))))
    }

    /// Get a market by slug. Uses TTL cache.
    pub async fn get_by_slug(&self, slug: &str) -> Result<Market, SdkError> {
        {
            let index = self.client.slug_index.read().await;
            if let Some(pubkey) = index.get(slug) {
                let cache = self.client.market_cache.read().await;
                if let Some((market, fetched_at)) = cache.get(pubkey) {
                    if fetched_at.elapsed() < self.client.market_cache_ttl {
                        return Ok(market.clone());
                    }
                }
            }
        }

        let resp = self.client.http.get_market_by_slug(slug).await?;
        let market: Market = resp
            .try_into()
            .map_err(|e: market::ValidationError| SdkError::Validation(e.to_string()))?;
        self.cache_market(&market).await;
        Ok(market)
    }

    /// Search markets.
    pub async fn search(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<MarketSearchResult>, SdkError> {
        Ok(self.client.http.search_markets(query, limit).await?)
    }

    /// Get featured markets.
    pub async fn featured(&self) -> Result<Vec<MarketSearchResult>, SdkError> {
        Ok(self.client.http.get_featured_markets().await?)
    }

    /// Invalidate a cached market by pubkey.
    pub async fn invalidate(&self, pubkey: &str) {
        self.client.market_cache.write().await.remove(pubkey);
    }

    /// Clear all market caches.
    pub async fn clear_cache(&self) {
        self.client.market_cache.write().await.clear();
        self.client.slug_index.write().await.clear();
    }

    async fn cache_market(&self, market: &Market) {
        let pubkey = market.pubkey.to_string();
        self.client
            .market_cache
            .write()
            .await
            .insert(pubkey.clone(), (market.clone(), Instant::now()));
        self.client
            .slug_index
            .write()
            .await
            .insert(market.slug.clone(), pubkey);
    }
}
