//! Market domain — market types, validation, conversion.

pub mod client;
mod convert;
pub mod outcome;
pub mod tokens;
pub mod wire;

use crate::domain::orderbook;
use crate::shared::{OrderBookId, PubkeyStr};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ─── Status ──────────────────────────────────────────────────────────────────

/// Market lifecycle status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub banner_image_url: String,
    pub icon_url: String,
    pub featured_rank: Option<i16>,
    pub volume: Decimal,
    pub slug: String,
    pub status: Status,
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
    pub winning_outcome: Option<i16>,
    pub description: String,
    pub definition: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub deposit_assets: Vec<self::tokens::DepositAsset>,
    pub conditional_tokens: Vec<self::tokens::ConditionalToken>,
    pub outcomes: Vec<self::outcome::Outcome>,
    pub orderbook_pairs: Vec<orderbook::OrderBookPair>,
    pub orderbook_ids: Vec<OrderBookId>,
    pub token_metadata: HashMap<PubkeyStr, self::tokens::TokenMetadata>,
}

// ─── Validation ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ValidationError {
    Multiple(String, Vec<ValidationError>),
    MarketNameMissing,
    MissingThumbnailImage,
    MissingBannerImage,
    InvalidStatus,
    MissingDescription,
    MissingDefinition,
    MissingSlug,
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
            ValidationError::MissingDescription => write!(f, "Missing description"),
            ValidationError::MissingDefinition => write!(f, "Missing definition"),
            ValidationError::MissingSlug => write!(f, "Missing slug"),
            ValidationError::MissingBannerImage => write!(f, "Missing banner image"),
            ValidationError::MissingThumbnailImage => write!(f, "Missing thumbnail image"),
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

