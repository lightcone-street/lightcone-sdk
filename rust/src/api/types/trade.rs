//! Trade-related types for the Lightcone REST API.

use serde::{Deserialize, Serialize};

/// Executed trade information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Trade ID
    pub id: i64,
    /// Orderbook ID
    pub orderbook_id: String,
    /// Taker's pubkey
    pub taker_pubkey: String,
    /// Maker's pubkey
    pub maker_pubkey: String,
    /// Trade side ("BID" or "ASK")
    pub side: String,
    /// Trade size as decimal string
    pub size: String,
    /// Trade price as decimal string
    pub price: String,
    /// Taker fee as decimal string
    pub taker_fee: String,
    /// Maker fee as decimal string
    pub maker_fee: String,
    /// Execution timestamp (milliseconds since epoch)
    pub executed_at: i64,
}

/// Query parameters for GET /api/trades.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradesParams {
    /// Orderbook identifier (required)
    pub orderbook_id: String,
    /// Filter by user pubkey
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_pubkey: Option<String>,
    /// Start timestamp (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    /// End timestamp (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
    /// Pagination cursor (trade ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<i64>,
    /// Max results (1-500)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl TradesParams {
    /// Create new params with required orderbook_id.
    pub fn new(orderbook_id: impl Into<String>) -> Self {
        Self {
            orderbook_id: orderbook_id.into(),
            ..Default::default()
        }
    }

    /// Set user pubkey filter.
    pub fn with_user(mut self, user_pubkey: impl Into<String>) -> Self {
        self.user_pubkey = Some(user_pubkey.into());
        self
    }

    /// Set time range.
    pub fn with_time_range(mut self, from: i64, to: i64) -> Self {
        self.from = Some(from);
        self.to = Some(to);
        self
    }

    /// Set pagination cursor.
    pub fn with_cursor(mut self, cursor: i64) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Set result limit.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Response for GET /api/trades.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesResponse {
    /// Orderbook ID
    pub orderbook_id: String,
    /// Trade list
    pub trades: Vec<Trade>,
    /// Next pagination cursor
    pub next_cursor: Option<i64>,
    /// Whether more results exist
    pub has_more: bool,
}
