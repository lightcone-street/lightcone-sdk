//! Trades sub-client — trade history queries.

use crate::client::LightconeClient;
use crate::domain::trade::Trade;
use crate::domain::trade::wire::TradesResponse;
use crate::error::SdkError;
use crate::http::RetryPolicy;

pub struct Trades<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Trades<'a> {
    pub async fn get(
        &self,
        orderbook_id: &str,
        limit: Option<u32>,
        before: Option<&str>,
    ) -> Result<Vec<Trade>, SdkError> {
        let mut url = format!(
            "{}/api/trades?orderbook_id={}",
            self.client.http.base_url(),
            orderbook_id
        );
        if let Some(l) = limit {
            url = format!("{}&limit={}", url, l);
        }
        if let Some(b) = before {
            url = format!("{}&before={}", url, b);
        }

        let resp: TradesResponse = self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?;

        Ok(resp.trades.into_iter().map(Trade::from).collect())
    }
}
