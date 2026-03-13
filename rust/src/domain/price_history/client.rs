//! Price history sub-client — OHLCV queries.

use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::shared::Resolution;

pub struct PriceHistoryClient<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> PriceHistoryClient<'a> {
    /// `from` and `to` are Unix timestamps in milliseconds.
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
            url = format!("{}&from={}", url, ensure_unix_milliseconds("from", f)?);
        }
        if let Some(t) = to {
            url = format!("{}&to={}", url, ensure_unix_milliseconds("to", t)?);
        }

        Ok(self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?)
    }
}

fn ensure_unix_milliseconds(name: &str, value: u64) -> Result<u64, SdkError> {
    if value < 10_000_000_000 {
        return Err(SdkError::Validation(format!(
            "{name} must be a Unix timestamp in milliseconds, not seconds"
        )));
    }
    Ok(value)
}
