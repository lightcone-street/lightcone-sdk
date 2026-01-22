//! Market-related types for the Lightcone REST API.

use serde::{Deserialize, Serialize};

/// Market status enum matching the API specification.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApiMarketStatus {
    /// Market created but not activated
    #[default]
    #[serde(rename = "Pending")]
    Pending,
    /// Market accepting orders
    #[serde(rename = "Active")]
    Active,
    /// Market has a resolved outcome
    #[serde(rename = "Settled")]
    Settled,
}

/// Outcome information for a market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    /// Outcome index (0-based)
    pub index: u32,
    /// Outcome name
    pub name: String,
    /// Optional thumbnail URL
    pub thumbnail_url: Option<String>,
}

/// Orderbook summary embedded in market response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSummary {
    /// Orderbook identifier
    pub orderbook_id: String,
    /// Market pubkey
    pub market_pubkey: String,
    /// Base token address
    pub base_token: String,
    /// Quote token address
    pub quote_token: String,
    /// Tick size for price granularity
    pub tick_size: u64,
    /// Creation timestamp
    pub created_at: String,
}

/// Conditional token information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalToken {
    /// Database ID
    pub id: i64,
    /// Outcome index this token represents
    pub outcome_index: u32,
    /// Token mint address
    pub token_address: String,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token metadata URI
    pub uri: Option<String>,
    /// Display name for UI
    pub display_name: String,
    /// Outcome name
    pub outcome: String,
    /// Associated deposit symbol
    pub deposit_symbol: String,
    /// Short name for display
    pub short_name: String,
    /// Token description
    pub description: Option<String>,
    /// Icon URL
    pub icon_url: Option<String>,
    /// Metadata URI
    pub metadata_uri: Option<String>,
    /// Token decimals
    pub decimals: u8,
    /// Creation timestamp
    pub created_at: String,
}

/// Deposit asset information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositAsset {
    /// Display name for the asset
    pub display_name: String,
    /// Token symbol
    pub token_symbol: String,
    /// Short symbol
    pub symbol: String,
    /// Deposit asset mint address
    pub deposit_asset: String,
    /// Database ID
    pub id: i64,
    /// Associated market pubkey
    pub market_pubkey: String,
    /// Vault address
    pub vault: String,
    /// Number of outcomes
    pub num_outcomes: u32,
    /// Asset description
    pub description: Option<String>,
    /// Icon URL
    pub icon_url: Option<String>,
    /// Metadata URI
    pub metadata_uri: Option<String>,
    /// Token decimals
    pub decimals: u8,
    /// Conditional tokens for each outcome
    pub conditional_tokens: Vec<ConditionalToken>,
    /// Creation timestamp
    pub created_at: String,
}

/// Market information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    /// Market name
    pub market_name: String,
    /// URL-friendly slug
    pub slug: String,
    /// Market description
    pub description: String,
    /// Market definition/rules
    pub definition: String,
    /// Possible outcomes
    pub outcomes: Vec<Outcome>,
    /// Banner image URL
    pub banner_image_url: Option<String>,
    /// Thumbnail URL
    pub thumbnail_url: Option<String>,
    /// Market category
    pub category: Option<String>,
    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,
    /// Featured rank (0 = not featured)
    #[serde(default)]
    pub featured_rank: i32,
    /// Market PDA address
    pub market_pubkey: String,
    /// Market ID
    pub market_id: u64,
    /// Oracle address
    pub oracle: String,
    /// Question ID
    pub question_id: String,
    /// Condition ID
    pub condition_id: String,
    /// Current market status
    pub market_status: ApiMarketStatus,
    /// Winning outcome index (if settled)
    #[serde(default)]
    pub winning_outcome: u32,
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
    /// Total count
    pub total: u64,
}

/// Response for GET /api/markets/{market_pubkey}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketInfoResponse {
    /// Market details
    pub market: Market,
    /// Deposit assets
    pub deposit_assets: Vec<DepositAsset>,
    /// Count of deposit assets
    pub deposit_asset_count: u64,
}

/// Response for GET /api/markets/{market_pubkey}/deposit-assets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositAssetsResponse {
    /// Market pubkey
    pub market_pubkey: String,
    /// Deposit assets
    pub deposit_assets: Vec<DepositAsset>,
    /// Total count
    pub total: u64,
}
