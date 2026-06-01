//! Price history sub-client — OHLCV queries.

use crate::client::LightconeClient;
use crate::domain::price_history::wire::{
    DepositAssetPricesSnapshotResponse, DepositPriceHistoryResponse, OrderbookPriceHistoryResponse,
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
        let url = format!("{}/api/price-history", self.client.http.base_url());
        let mut params = vec![
            ("orderbook_id", orderbook_id.to_string()),
            ("resolution", query.resolution.as_str().to_string()),
        ];
        if let Some(from) = query.from {
            params.push(("from", ensure_unix_milliseconds("from", from)?.to_string()));
        }
        if let Some(to) = query.to {
            params.push(("to", ensure_unix_milliseconds("to", to)?.to_string()));
        }
        if let Some(cursor) = query.cursor {
            params.push((
                "cursor",
                ensure_unix_milliseconds("cursor", cursor)?.to_string(),
            ));
        }
        if let Some(limit) = query.limit {
            params.push(("limit", ensure_page_limit(limit)?.to_string()));
        }
        if query.include_ohlcv {
            params.push(("include_ohlcv", "true".to_string()));
        }

        self.client
            .http
            .get_with_query(&url, &params, RetryPolicy::Idempotent)
            .await
    }

    /// Get deposit-token price history from the same REST endpoint.
    pub async fn get_deposit_asset(
        &self,
        deposit_asset: &str,
        query: DepositPriceHistoryQuery,
    ) -> Result<DepositPriceHistoryResponse, SdkError> {
        let url = format!("{}/api/price-history", self.client.http.base_url());
        let mut params = vec![
            ("deposit_asset", deposit_asset.to_string()),
            ("resolution", query.resolution.as_str().to_string()),
        ];
        if let Some(from) = query.from {
            params.push(("from", ensure_unix_milliseconds("from", from)?.to_string()));
        }
        if let Some(to) = query.to {
            params.push(("to", ensure_unix_milliseconds("to", to)?.to_string()));
        }
        if let Some(cursor) = query.cursor {
            params.push((
                "cursor",
                ensure_unix_milliseconds("cursor", cursor)?.to_string(),
            ));
        }
        if let Some(limit) = query.limit {
            params.push(("limit", ensure_page_limit(limit)?.to_string()));
        }

        self.client
            .http
            .get_with_query(&url, &params, RetryPolicy::Idempotent)
            .await
    }

    /// Snapshot of current prices for every active mint in `global_deposit_tokens`.
    ///
    /// No params. Returns a map of mint -> price (Decimal-as-string). The
    /// backend prefers the live tick from `deposit_token_prices` and falls
    /// back to the most recent 1m candle close. Assets with neither are
    /// silently absent.
    ///
    /// For live updates, subscribe to `MessageOut::subscribe_deposit_asset_price`
    /// per asset.
    pub async fn get_deposit_asset_prices_snapshot(
        &self,
    ) -> Result<DepositAssetPricesSnapshotResponse, SdkError> {
        let url = format!(
            "{}/api/deposit-asset-prices-snapshot",
            self.client.http.base_url()
        );
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
