//! Trades sub-client — trade history queries.

use crate::client::LightconeClient;
use crate::domain::trade::Trade;
use crate::error::SdkError;

/// Sub-client for trade operations.
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
        let resp = self
            .client
            .http
            .get_trades(orderbook_id, limit, before)
            .await?;
        Ok(resp.trades.into_iter().map(Trade::from).collect())
    }
}
