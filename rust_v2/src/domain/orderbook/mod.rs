//! Orderbook domain — orderbook pairs, validation, impact calculations.

pub mod client;
mod convert;
pub mod state;
pub mod ticker;
pub mod wire;

pub use ticker::TickerData;

use crate::domain::market::tokens;
use crate::shared::{OrderBookId, PubkeyStr};
use chrono::{DateTime, Utc};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A pair of conditional tokens that can be traded against each other.
///
/// There are multiple orderbook pairs per market because each pair is specific
/// to a base + quote + condition combination.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderBookPair {
    pub id: i32,
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub base: tokens::ConditionalToken,
    pub quote: tokens::ConditionalToken,
    pub outcome_index: i16,
    pub tick_size: i64,
    pub total_bids: i32,
    pub total_asks: i32,
    pub last_trade_price: Option<Decimal>,
    pub last_trade_time: Option<DateTime<Utc>>,
    pub active: bool,
}

impl OrderBookPair {
    /// Price impact as percentage relative to a deposit asset price.
    pub fn impact_pct(&self, deposit_price: Decimal) -> (f64, &'static str) {
        if deposit_price == Decimal::ZERO {
            return (0.0, "");
        }

        if let Some(conditional) = self.last_trade_price {
            let val = ((conditional - deposit_price) / deposit_price) * Decimal::from(100);
            let sign = if val > Decimal::ZERO { "+" } else { "" };
            (val.to_f64().unwrap_or(0.0), sign)
        } else {
            (0.0, "")
        }
    }

    /// Full impact calculation with sign, percentage, and dollar difference.
    pub fn impact(
        &self,
        deposit_asset_price: Decimal,
        conditional_price: Decimal,
    ) -> OutcomeImpact {
        if deposit_asset_price == Decimal::ZERO {
            return OutcomeImpact::default();
        }

        let pct_decimal = ((conditional_price - deposit_asset_price) / deposit_asset_price)
            * Decimal::from(100);
        let pct = pct_decimal.to_f64().unwrap_or(0.0);
        let sign = String::from(if pct > 0.0 { "+" } else { "-" });

        OutcomeImpact {
            sign,
            is_positive: pct > 0.0,
            pct: pct.abs(),
            dollar: (conditional_price - deposit_asset_price).abs(),
        }
    }
}

/// Calculated impact of a conditional token's price vs its deposit asset.
#[derive(Debug, Clone, PartialEq)]
pub struct OutcomeImpact {
    pub sign: String,
    pub pct: f64,
    pub dollar: Decimal,
    pub is_positive: bool,
}

impl Default for OutcomeImpact {
    fn default() -> Self {
        Self {
            sign: String::new(),
            pct: 0.0,
            dollar: Decimal::ZERO,
            is_positive: false,
        }
    }
}

// ─── Validation ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum OrderBookValidationError {
    Multiple(String, Vec<OrderBookValidationError>),
    BaseTokenNotFound(String),
    QuoteTokenNotFound(String),
}

impl fmt::Display for OrderBookValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderBookValidationError::Multiple(id, errors) => {
                writeln!(f, "OrderBook validation errors ({id}):")?;
                for err in errors {
                    writeln!(f, "  - {}", err)?;
                }
                Ok(())
            }
            OrderBookValidationError::BaseTokenNotFound(m) => {
                write!(f, "Base token not found: {m}")
            }
            OrderBookValidationError::QuoteTokenNotFound(m) => {
                write!(f, "Quote token not found: {m}")
            }
        }
    }
}

impl std::error::Error for OrderBookValidationError {}

