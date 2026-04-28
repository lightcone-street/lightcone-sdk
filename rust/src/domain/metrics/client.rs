//! Metrics sub-client — platform, market, orderbook, category, deposit-token,
//! leaderboard, and history metrics.

use crate::client::LightconeClient;
use crate::domain::metrics::wire::{
    CategoriesMetrics, CategoryMetricsQuery, CategoryVolumeMetrics, DepositTokensMetrics,
    Leaderboard, MarketDetailMetrics, MarketMetricsQuery, MarketsMetrics, MarketsMetricsQuery,
    MetricsHistory, MetricsHistoryQuery, OrderbookMetricsQuery, OrderbookTickersResponse,
    OrderbookVolumeMetrics, PlatformMetrics,
};
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::shared::{OrderBookId, PubkeyStr};

fn append_query(url: &mut String, qs: &str) {
    if !qs.is_empty() {
        url.push(if url.contains('?') { '&' } else { '?' });
        url.push_str(qs);
    }
}

/// Metrics sub-client. Obtain via [`LightconeClient::metrics`].
pub struct Metrics<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Metrics<'a> {
    /// Fetch platform-wide metrics: total volume, trader counts, active market/orderbook
    /// counts, and per-deposit-token breakdowns.
    ///
    /// `GET /api/metrics/platform`
    pub async fn platform(&self) -> Result<PlatformMetrics, SdkError> {
        let url = format!("{}/api/metrics/platform", self.client.http.base_url());
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// List metrics for all active markets.
    ///
    /// `GET /api/metrics/markets`
    pub async fn markets(&self, query: &MarketsMetricsQuery) -> Result<MarketsMetrics, SdkError> {
        let mut url = format!("{}/api/metrics/markets", self.client.http.base_url());
        if let Ok(qs) = serde_urlencoded::to_string(query) {
            append_query(&mut url, &qs);
        }
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Fetch detailed metrics for a single market — including per-outcome, per-orderbook,
    /// and per-deposit-token breakdowns.
    ///
    /// `GET /api/metrics/markets/{market_pubkey}`
    pub async fn market(
        &self,
        market_pubkey: &PubkeyStr,
        query: &MarketMetricsQuery,
    ) -> Result<MarketDetailMetrics, SdkError> {
        let mut url = format!(
            "{}/api/metrics/markets/{}",
            self.client.http.base_url(),
            urlencoding::encode(market_pubkey.as_str())
        );
        if let Ok(qs) = serde_urlencoded::to_string(query) {
            append_query(&mut url, &qs);
        }
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Batch BBO + midpoint per active orderbook (same shape as the WS
    /// `Ticker` stream, delivered in one REST call). Optionally filter to
    /// orderbooks whose base conditional-token is backed by `deposit_asset`.
    /// Prices per orderbook are scaled using that orderbook's own decimals.
    ///
    /// `GET /api/metrics/orderbooks/tickers[?deposit_asset=<mint>]`
    pub async fn orderbook_tickers(
        &self,
        deposit_asset: Option<&str>,
    ) -> Result<OrderbookTickersResponse, SdkError> {
        let mut url = format!(
            "{}/api/metrics/orderbooks/tickers",
            self.client.http.base_url()
        );
        if let Some(mint) = deposit_asset.map(str::trim).filter(|s| !s.is_empty()) {
            append_query(
                &mut url,
                &format!("deposit_asset={}", urlencoding::encode(mint)),
            );
        }
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Fetch metrics for a single orderbook, broken down by base/quote/USD volume.
    ///
    /// `GET /api/metrics/orderbooks/{orderbook_id}`
    pub async fn orderbook(
        &self,
        orderbook_id: &OrderBookId,
        query: &OrderbookMetricsQuery,
    ) -> Result<OrderbookVolumeMetrics, SdkError> {
        let mut url = format!(
            "{}/api/metrics/orderbooks/{}",
            self.client.http.base_url(),
            urlencoding::encode(orderbook_id.as_str())
        );
        if let Ok(qs) = serde_urlencoded::to_string(query) {
            append_query(&mut url, &qs);
        }
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// List metrics for every market category (e.g. Politics, Sports).
    ///
    /// `GET /api/metrics/categories`
    pub async fn categories(&self) -> Result<CategoriesMetrics, SdkError> {
        let url = format!("{}/api/metrics/categories", self.client.http.base_url());
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Fetch metrics for a single category.
    ///
    /// `GET /api/metrics/categories/{category}`
    pub async fn category(
        &self,
        category: &str,
        query: &CategoryMetricsQuery,
    ) -> Result<CategoryVolumeMetrics, SdkError> {
        let mut url = format!(
            "{}/api/metrics/categories/{}",
            self.client.http.base_url(),
            urlencoding::encode(category)
        );
        if let Ok(qs) = serde_urlencoded::to_string(query) {
            append_query(&mut url, &qs);
        }
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// List metrics per deposit token across the entire platform.
    ///
    /// `GET /api/metrics/deposit-tokens`
    pub async fn deposit_tokens(&self) -> Result<DepositTokensMetrics, SdkError> {
        let url = format!("{}/api/metrics/deposit-tokens", self.client.http.base_url());
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Fetch the market leaderboard (top markets by 24h volume).
    ///
    /// `GET /api/metrics/leaderboard/markets`
    pub async fn leaderboard(&self, limit: Option<u32>) -> Result<Leaderboard, SdkError> {
        let mut url = format!(
            "{}/api/metrics/leaderboard/markets",
            self.client.http.base_url()
        );
        if let Some(limit) = limit {
            append_query(&mut url, &format!("limit={limit}"));
        }
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Fetch a time-series of volume buckets for the given scope and scope key.
    ///
    /// `scope` is one of `"orderbook" | "market" | "category" | "deposit_token" | "platform"`.
    /// `scope_key` is the corresponding identifier (e.g. an orderbook ID for
    /// `scope = "orderbook"`). `MetricsHistoryQuery::default()` yields `"1h"` resolution
    /// with no time bounds.
    ///
    /// `GET /api/metrics/history/{scope}/{scope_key}`
    pub async fn history(
        &self,
        scope: &str,
        scope_key: &str,
        query: &MetricsHistoryQuery,
    ) -> Result<MetricsHistory, SdkError> {
        let mut url = format!(
            "{}/api/metrics/history/{}/{}",
            self.client.http.base_url(),
            urlencoding::encode(scope),
            urlencoding::encode(scope_key)
        );
        if let Ok(qs) = serde_urlencoded::to_string(query) {
            append_query(&mut url, &qs);
        }
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }
}
