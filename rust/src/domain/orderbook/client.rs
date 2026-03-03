//! Orderbooks sub-client — depth, decimals.

use crate::client::LightconeClient;
use crate::domain::orderbook::wire::{DecimalsResponse, OrderbookDepthResponse};
use crate::error::SdkError;
use crate::http::RetryPolicy;

pub struct Orderbooks<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Orderbooks<'a> {
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

    /// Get orderbook decimals (memoized — rarely changes).
    pub async fn decimals(&self, orderbook_id: &str) -> Result<DecimalsResponse, SdkError> {
        {
            let cache = self.client.decimals_cache.read().await;
            if let Some(d) = cache.get(orderbook_id) {
                return Ok(d.clone());
            }
        }

        let url = format!(
            "{}/api/orderbooks/{}/decimals",
            self.client.http.base_url(),
            orderbook_id
        );
        let resp: DecimalsResponse = self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?;

        self.client
            .decimals_cache
            .write()
            .await
            .insert(orderbook_id.to_string(), resp.clone());
        Ok(resp)
    }

    pub async fn clear_cache(&self) {
        self.client.decimals_cache.write().await.clear();
    }
}
