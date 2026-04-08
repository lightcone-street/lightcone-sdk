//! Price history sub-client — OHLCV queries.

use crate::client::LightconeClient;
use crate::domain::price_history::wire::{
    DepositPriceHistoryResponse, OrderbookPriceHistoryResponse,
};
use crate::domain::price_history::{
    DepositPriceHistoryQuery, LineData, OrderbookPriceHistoryQuery,
};
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
    ) -> Result<OrderbookPriceHistoryResponse, SdkError> {
        self.get_with_query(
            orderbook_id,
            OrderbookPriceHistoryQuery {
                resolution,
                from,
                to,
                ..OrderbookPriceHistoryQuery::default()
            },
        )
        .await
    }

    /// Get orderbook price history with pagination and OHLCV options.
    pub async fn get_with_query(
        &self,
        orderbook_id: &str,
        query: OrderbookPriceHistoryQuery,
    ) -> Result<OrderbookPriceHistoryResponse, SdkError> {
        let mut url = format!(
            "{}/api/price-history?orderbook_id={}&resolution={}",
            self.client.http.base_url(),
            orderbook_id,
            query.resolution.as_str()
        );
        if let Some(f) = query.from {
            url = format!("{}&from={}", url, ensure_unix_milliseconds("from", f)?);
        }
        if let Some(t) = query.to {
            url = format!("{}&to={}", url, ensure_unix_milliseconds("to", t)?);
        }
        if let Some(cursor) = query.cursor {
            url = format!(
                "{}&cursor={}",
                url,
                ensure_unix_milliseconds("cursor", cursor)?
            );
        }
        if let Some(limit) = query.limit {
            url = format!("{}&limit={}", url, ensure_page_limit(limit)?);
        }
        if query.include_ohlcv {
            url = format!("{}&include_ohlcv=true", url);
        }

        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Get deposit-token price history from the same REST endpoint.
    pub async fn get_deposit_asset(
        &self,
        deposit_asset: &str,
        query: DepositPriceHistoryQuery,
    ) -> Result<DepositPriceHistoryResponse, SdkError> {
        let mut url = format!(
            "{}/api/price-history?deposit_asset={}&resolution={}",
            self.client.http.base_url(),
            deposit_asset,
            query.resolution.as_str()
        );
        if let Some(f) = query.from {
            url = format!("{}&from={}", url, ensure_unix_milliseconds("from", f)?);
        }
        if let Some(t) = query.to {
            url = format!("{}&to={}", url, ensure_unix_milliseconds("to", t)?);
        }
        if let Some(cursor) = query.cursor {
            url = format!(
                "{}&cursor={}",
                url,
                ensure_unix_milliseconds("cursor", cursor)?
            );
        }
        if let Some(limit) = query.limit {
            url = format!("{}&limit={}", url, ensure_page_limit(limit)?);
        }

        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Get simplified midpoint line data for charting.
    pub async fn get_line_data(
        &self,
        orderbook_id: &str,
        resolution: Resolution,
        from: Option<u64>,
        to: Option<u64>,
        cursor: Option<u64>,
        limit: Option<usize>,
    ) -> Result<Vec<LineData>, SdkError> {
        let response = self
            .get_with_query(
                orderbook_id,
                OrderbookPriceHistoryQuery {
                    resolution,
                    from,
                    to,
                    cursor,
                    limit,
                    include_ohlcv: false,
                },
            )
            .await?;

        Ok(response.prices.into_iter().map(LineData::from).collect())
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

fn ensure_page_limit(value: usize) -> Result<usize, SdkError> {
    if !(1..=1000).contains(&value) {
        return Err(SdkError::Validation(
            "limit must be an integer between 1 and 1000".to_string(),
        ));
    }
    Ok(value)
}
