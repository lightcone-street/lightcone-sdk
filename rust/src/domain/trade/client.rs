//! Trades sub-client — trade history queries.

use crate::client::LightconeClient;
use crate::domain::trade::wire::{MarketTradesResponse, TradesResponse};
use crate::domain::trade::{Trade, TradesPage};
use crate::error::SdkError;
use crate::http::RetryPolicy;

pub struct Trades<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Trades<'a> {
    /// Get trades for an orderbook.
    ///
    /// `cursor` is a numeric REST row id for pagination — pass `next_cursor`
    /// from a previous response to get the next page.
    pub async fn get(
        &self,
        orderbook_id: &str,
        limit: Option<u32>,
        cursor: Option<i64>,
    ) -> Result<TradesPage, SdkError> {
        let mut url = format!(
            "{}/api/trades?orderbook_id={}",
            self.client.http.base_url(),
            orderbook_id
        );
        if let Some(l) = limit {
            url = format!("{}&limit={}", url, l);
        }
        if let Some(c) = cursor {
            url = format!("{}&cursor={}", url, c);
        }

        let resp: TradesResponse = self.client.http.get(&url, RetryPolicy::Idempotent).await?;

        Ok(TradesPage {
            trades: resp.trades.into_iter().map(Trade::from).collect(),
            next_cursor: resp.next_cursor,
            has_more: resp.has_more,
        })
    }

    /// Get trades for all orderbooks in a market.
    ///
    /// Returns trades from every outcome interleaved by time.
    /// `cursor` is a numeric REST row id for pagination — pass `next_cursor`
    /// from a previous response to get the next page.
    pub async fn get_by_market(
        &self,
        market_pubkey: &str,
        limit: Option<u32>,
        cursor: Option<i64>,
    ) -> Result<TradesPage, SdkError> {
        let mut url = format!(
            "{}/api/trades/market?market_pubkey={}",
            self.client.http.base_url(),
            market_pubkey
        );
        if let Some(l) = limit {
            url = format!("{}&limit={}", url, l);
        }
        if let Some(c) = cursor {
            url = format!("{}&cursor={}", url, c);
        }

        let resp: MarketTradesResponse =
            self.client.http.get(&url, RetryPolicy::Idempotent).await?;

        Ok(TradesPage {
            trades: resp.trades.into_iter().map(Trade::from).collect(),
            next_cursor: resp.next_cursor,
            has_more: resp.has_more,
        })
    }
}
