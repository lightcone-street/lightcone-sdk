//! Price history sub-client — OHLCV queries.

use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::shared::Resolution;

/// Sub-client for price history operations.
pub struct PriceHistoryClient<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> PriceHistoryClient<'a> {
    pub async fn get(
        &self,
        orderbook_id: &str,
        resolution: Resolution,
        from: Option<u64>,
        to: Option<u64>,
    ) -> Result<serde_json::Value, SdkError> {
        Ok(self
            .client
            .http
            .get_price_history(orderbook_id, resolution, from, to)
            .await?)
    }
}
