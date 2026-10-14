#![doc = include_str!("README.md")]

pub mod client;
mod convert;
pub mod outcome;
pub mod tokens;
pub mod wire;

pub use self::tokens::{DepositAssetPair, GlobalDepositAsset};
pub use self::wire::{MarketResolutionKind, MarketResolutionPayout, MarketResolutionResponse};

use crate::domain::orderbook;
use crate::shared::{OrderBookId, PubkeyStr};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ─── Icon URL resolution ────────────────────────────────────────────────────

/// Resolve three icon URL quality variants with cross-fallback.
/// Returns `None` if all three inputs are `None`.
pub fn resolve_icon_urls(
    low: Option<String>,
    medium: Option<String>,
    high: Option<String>,
) -> Option<(String, String, String)> {
    // Return None when all three are missing.
    let fallback = low.as_ref().or(medium.as_ref()).or(high.as_ref())?.clone();
    let resolved_low = low
        .clone()
        .or_else(|| medium.clone())
        .or_else(|| high.clone())
        .unwrap_or_else(|| fallback.clone());
    let resolved_medium = medium
        .clone()
        .or_else(|| low.clone())
        .or_else(|| high.clone())
        .unwrap_or_else(|| fallback.clone());
    let resolved_high = high.or(medium).or(low).unwrap_or(fallback);
    Some((resolved_low, resolved_medium, resolved_high))
}

// ─── Status ──────────────────────────────────────────────────────────────────

/// Market lifecycle status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    Pending,
    Active,
    Resolved,
    Cancelled,
}

impl Status {
    pub fn as_str(&self) -> &str {
        match self {
            Status::Pending => "Pending",
            Status::Active => "Active",
            Status::Resolved => "Resolved",
            Status::Cancelled => "Cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Pending" => Some(Status::Pending),
            "Active" => Some(Status::Active),
            "Resolved" => Some(Status::Resolved),
            "Cancelled" => Some(Status::Cancelled),
            _ => None,
        }
    }
}

// ─── Market ──────────────────────────────────────────────────────────────────

/// A fully validated market with all nested domain types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Market {
    pub id: i64,
    pub pubkey: PubkeyStr,
    pub name: String,
    pub banner_image_url_low: Option<String>,
    pub banner_image_url_medium: Option<String>,
    pub banner_image_url_high: Option<String>,
    pub icon_url_low: String,
    pub icon_url_medium: String,
    pub icon_url_high: String,
    pub featured_rank: Option<i16>,
    pub slug: String,
    pub status: Status,
    /// Maker fee in basis points (1 bps = 0.01%); negative means a rebate.
    /// Set per market at creation, admin-updatable on chain.
    pub maker_fee_bps: i16,
    /// Taker fee in basis points (1 bps = 0.01%); negative means a rebate.
    /// Set per market at creation, admin-updatable on chain.
    pub taker_fee_bps: i16,
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
    /// Resolution deadline as a Unix timestamp in milliseconds.
    pub resolution_by: Option<i64>,
    pub resolution: Option<MarketResolutionResponse>,
    pub description: Option<String>,
    pub definition: String,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub tags: Vec<String>,
    pub num_outcomes: u8,
    pub deposit_assets: Vec<self::tokens::DepositAsset>,
    /// Unique base/quote deposit-asset pairs derived from `orderbook_pairs`
    /// during wire→domain conversion. Deduplicated by `(base, quote)` pubkey.
    pub deposit_asset_pairs: Vec<self::tokens::DepositAssetPair>,
    pub conditional_tokens: Vec<self::tokens::ConditionalToken>,
    pub outcomes: Vec<self::outcome::Outcome>,
    pub orderbook_pairs: Vec<orderbook::OrderBookPair>,
    pub orderbook_ids: Vec<OrderBookId>,
    pub token_metadata: HashMap<PubkeyStr, self::tokens::TokenMetadata>,
}

impl Market {
    pub fn is_resolved(&self) -> bool {
        self.resolution.is_some()
    }

    pub fn single_winning_outcome(&self) -> Option<i16> {
        self.resolution
            .as_ref()
            .and_then(|resolution| resolution.single_winning_outcome)
    }

    pub fn has_single_winning_outcome(&self) -> bool {
        self.single_winning_outcome().is_some()
    }
}

// ─── Validation ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ValidationError {
    Multiple(String, Vec<ValidationError>),
    MarketNameMissing,
    MissingIconUrl,
    InvalidStatus,
    MissingDefinition,
    MissingSlug,
    InvalidOutcomeCount(i16),
    InconsistentOutcomeCounts(Vec<i16>),
    MissingDepositAssetPairs,
    Token(self::tokens::TokenValidationError),
    Outcome(self::outcome::OutcomeValidationError),
    OrderBook(orderbook::OrderBookValidationError),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::Multiple(pda, errors) => {
                writeln!(f, "Market validation errors ({pda}):")?;
                for err in errors {
                    writeln!(f, "  - {}", err)?;
                }
                Ok(())
            }
            ValidationError::MarketNameMissing => write!(f, "Missing name"),
            ValidationError::InvalidStatus => write!(f, "Invalid status"),
            ValidationError::MissingDefinition => write!(f, "Missing definition"),
            ValidationError::MissingSlug => write!(f, "Missing slug"),
            ValidationError::InvalidOutcomeCount(count) => {
                write!(f, "Invalid outcome count: {count}")
            }
            ValidationError::InconsistentOutcomeCounts(counts) => {
                write!(f, "Inconsistent outcome counts: {counts:?}")
            }
            ValidationError::MissingIconUrl => write!(f, "Missing icon URL"),
            ValidationError::MissingDepositAssetPairs => write!(f, "Missing deposit asset pairs"),
            ValidationError::Token(err) => write!(f, "Token: {}", err),
            ValidationError::Outcome(err) => write!(f, "Outcome: {}", err),
            ValidationError::OrderBook(err) => write!(f, "OrderBook: {}", err),
        }
    }
}

impl std::error::Error for ValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ValidationError::Token(e) => Some(e),
            ValidationError::Outcome(e) => Some(e),
            ValidationError::OrderBook(e) => Some(e),
            _ => None,
        }
    }
}
