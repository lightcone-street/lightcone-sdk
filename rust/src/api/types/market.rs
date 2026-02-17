//! Market-related types for the Lightcone REST API.

use serde::{Deserialize, Serialize};

/// Outcome information for a market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    /// Outcome index (0-based)
    pub index: i16,
    /// Outcome name
    pub name: String,
    /// Optional icon URL
    #[serde(default)]
    pub icon_url: Option<String>,
}

/// Orderbook summary embedded in market response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSummary {
    /// Database ID
    pub id: i32,
    /// Market pubkey
    pub market_pubkey: String,
    /// Orderbook identifier
    pub orderbook_id: String,
    /// Base token address
    pub base_token: String,
    /// Quote token address
    pub quote_token: String,
    /// Outcome index this orderbook represents
    #[serde(default)]
    pub outcome_index: Option<i16>,
    /// Tick size for price granularity
    pub tick_size: i64,
    /// Total number of bid orders
    pub total_bids: i32,
    /// Total number of ask orders
    pub total_asks: i32,
    /// Last trade price as scaled decimal string
    #[serde(default)]
    pub last_trade_price: Option<String>,
    /// Last trade timestamp
    #[serde(default)]
    pub last_trade_time: Option<String>,
    /// Whether the orderbook is active
    pub active: bool,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

/// Conditional token information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalToken {
    /// Database ID
    pub id: i32,
    /// Outcome index this token represents
    pub outcome_index: i16,
    /// Token mint address
    pub token_address: String,
    /// Token name
    #[serde(default)]
    pub name: Option<String>,
    /// Token symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Token metadata URI
    #[serde(default)]
    pub uri: Option<String>,
    /// Display name for UI
    #[serde(default)]
    pub display_name: Option<String>,
    /// Outcome name
    #[serde(default)]
    pub outcome: Option<String>,
    /// Associated deposit symbol
    #[serde(default)]
    pub deposit_symbol: Option<String>,
    /// Short name for display
    #[serde(default)]
    pub short_name: Option<String>,
    /// Token description
    #[serde(default)]
    pub description: Option<String>,
    /// Icon URL
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Metadata URI
    #[serde(default)]
    pub metadata_uri: Option<String>,
    /// Token decimals
    #[serde(default)]
    pub decimals: Option<i16>,
    /// Creation timestamp
    pub created_at: String,
}

/// Deposit asset information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositAsset {
    /// Display name for the asset
    #[serde(default)]
    pub display_name: Option<String>,
    /// Token symbol
    #[serde(default)]
    pub token_symbol: Option<String>,
    /// Short symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Deposit asset mint address
    pub deposit_asset: String,
    /// Database ID
    pub id: i32,
    /// Associated market pubkey
    pub market_pubkey: String,
    /// Vault address
    pub vault: String,
    /// Number of outcomes
    pub num_outcomes: i16,
    /// Asset description
    #[serde(default)]
    pub description: Option<String>,
    /// Icon URL
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Metadata URI
    #[serde(default)]
    pub metadata_uri: Option<String>,
    /// Token decimals
    #[serde(default)]
    pub decimals: Option<i16>,
    /// Conditional tokens for each outcome
    #[serde(default)]
    pub conditional_mints: Vec<ConditionalToken>,
    /// Creation timestamp
    pub created_at: String,
}

/// Market information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    /// Market name
    #[serde(default)]
    pub market_name: Option<String>,
    /// URL-friendly slug
    #[serde(default)]
    pub slug: Option<String>,
    /// Market description
    #[serde(default)]
    pub description: Option<String>,
    /// Market definition/rules
    #[serde(default)]
    pub definition: Option<String>,
    /// Possible outcomes
    pub outcomes: Vec<Outcome>,
    /// Banner image URL
    #[serde(default)]
    pub banner_image_url: Option<String>,
    /// Icon URL
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Market category
    #[serde(default)]
    pub category: Option<String>,
    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,
    /// Featured rank (0 = not featured)
    #[serde(default)]
    pub featured_rank: i16,
    /// Market PDA address
    pub market_pubkey: String,
    /// Market ID
    pub market_id: i64,
    /// Oracle address
    pub oracle: String,
    /// Question ID
    pub question_id: String,
    /// Condition ID
    pub condition_id: String,
    /// Current market status (e.g. "Active", "Pending", "settled")
    pub market_status: String,
    /// Winning outcome index (if resolved)
    #[serde(default)]
    pub winning_outcome: Option<i16>,
    /// Whether market has a winning outcome
    #[serde(default)]
    pub has_winning_outcome: bool,
    /// Creation timestamp
    pub created_at: String,
    /// Activation timestamp
    pub activated_at: Option<String>,
    /// Settlement timestamp
    pub settled_at: Option<String>,
    /// Deposit assets for this market
    #[serde(default)]
    pub deposit_assets: Vec<DepositAsset>,
    /// Orderbooks for this market
    #[serde(default)]
    pub orderbooks: Vec<OrderbookSummary>,
}

/// Response for GET /api/markets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsResponse {
    /// List of markets
    pub markets: Vec<Market>,
    /// Cursor for next page (None when no more results)
    #[serde(default)]
    pub next_cursor: Option<i64>,
    /// Whether more results exist
    #[serde(default)]
    pub has_more: bool,
}

/// Response for GET /api/markets/{market_pubkey}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketInfoResponse {
    /// Market details
    pub market: Market,
    /// Deposit assets
    pub deposit_assets: Vec<DepositAsset>,
    /// Count of deposit assets
    pub deposit_asset_count: usize,
}

/// Response for GET /api/markets/{market_pubkey}/deposit-assets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositAssetsResponse {
    /// Market pubkey
    pub market_pubkey: String,
    /// Deposit assets
    pub deposit_assets: Vec<DepositAsset>,
    /// Total count
    pub total: usize,
}

/// Orderbook info returned in market search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOrderbook {
    /// Orderbook identifier
    pub orderbook_id: String,
    /// Outcome name (e.g. "Yes", "No")
    pub outcome_name: String,
    /// Outcome index
    pub outcome_index: i16,
    /// Deposit base asset mint address
    pub deposit_base_asset: String,
    /// Deposit quote asset mint address
    pub deposit_quote_asset: String,
    /// Base asset symbol
    pub deposit_base_symbol: String,
    /// Quote asset symbol
    pub deposit_quote_symbol: String,
    /// Base asset icon URL
    pub base_icon_url: String,
    /// Quote asset icon URL
    pub quote_icon_url: String,
    /// Conditional base token mint
    pub conditional_base_mint: String,
    /// Conditional quote token mint
    pub conditional_quote_mint: String,
    /// Latest midpoint price as scaled decimal string
    #[serde(default)]
    pub latest_mid_price: Option<String>,
}

/// Market search result returned by search and featured endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSearchResult {
    /// URL-friendly slug
    pub slug: String,
    /// Market name
    pub market_name: String,
    /// Market status string (e.g. "Active", "Resolved")
    pub market_status: String,
    /// Market category
    #[serde(default)]
    pub category: Option<String>,
    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,
    /// Featured rank (0 = not featured)
    #[serde(default)]
    pub featured_rank: i16,
    /// Market description
    #[serde(default)]
    pub description: Option<String>,
    /// Icon URL
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Orderbooks with pricing info
    #[serde(default)]
    pub orderbooks: Vec<SearchOrderbook>,
}
