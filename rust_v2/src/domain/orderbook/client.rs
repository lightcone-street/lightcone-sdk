//! Orderbooks sub-client — depth, decimals, cache.

use crate::client::LightconeClient;
use crate::domain::orderbook::wire::{DecimalsResponse, OrderbookDepthResponse};
use crate::error::SdkError;

/// Sub-client for orderbook operations.
pub struct Orderbooks<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Orderbooks<'a> {
    /// Get live orderbook depth (never cached — always fresh).
    pub async fn get(
        &self,
        orderbook_id: &str,
        depth: Option<u32>,
    ) -> Result<OrderbookDepthResponse, SdkError> {
        Ok(self.client.http.get_orderbook(orderbook_id, depth).await?)
    }

    /// Get orderbook decimals (persistently cached — rarely changes).
    pub async fn decimals(&self, orderbook_id: &str) -> Result<DecimalsResponse, SdkError> {
        {
            let cache = self.client.decimals_cache.read().await;
            if let Some(d) = cache.get(orderbook_id) {
                return Ok(d.clone());
            }
        }

        let resp = self.client.http.get_orderbook_decimals(orderbook_id).await?;
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
