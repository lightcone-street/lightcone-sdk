//! Price history sub-client — OHLCV queries.

use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::shared::Resolution;

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
        let mut url = format!(
            "{}/api/price-history?orderbook_id={}&resolution={}",
            self.client.http.base_url(),
            orderbook_id,
            resolution.as_str()
        );
        if let Some(f) = from {
            url = format!("{}&from={}", url, f);
        }
        if let Some(t) = to {
            url = format!("{}&to={}", url, t);
        }

        Ok(self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?)
    }
}
