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
        let url = format!("{}/api/trades", self.client.http.base_url());
        let mut query = vec![("orderbook_id", orderbook_id.to_string())];
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(cursor) = cursor {
            query.push(("cursor", cursor.to_string()));
        }

        let resp: TradesResponse = self
            .client
            .http
            .get_with_query(&url, &query, RetryPolicy::Idempotent)
            .await?;

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
        let url = format!("{}/api/trades/market", self.client.http.base_url());
        let mut query = vec![("market_pubkey", market_pubkey.to_string())];
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(cursor) = cursor {
            query.push(("cursor", cursor.to_string()));
        }

        let resp: MarketTradesResponse = self
            .client
            .http
            .get_with_query(&url, &query, RetryPolicy::Idempotent)
            .await?;

        Ok(TradesPage {
            trades: resp.trades.into_iter().map(Trade::from).collect(),
            next_cursor: resp.next_cursor,
            has_more: resp.has_more,
        })
    }
}
