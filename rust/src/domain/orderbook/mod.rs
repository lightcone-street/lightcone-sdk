#![doc = include_str!("README.md")]

pub mod client;
mod convert;
pub mod state;
pub mod ticker;
pub mod wire;

pub use ticker::TickerData;

use crate::domain::market::tokens::{self, Token};
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
    /// Derive scaling decimals from this pair's token metadata.
    ///
    /// This is the recommended way to get `OrderbookDecimals` — no REST call needed.
    pub fn decimals(&self) -> crate::shared::scaling::OrderbookDecimals {
        let base_decimals = self.base.decimals() as u8;
        let quote_decimals = self.quote.decimals() as u8;
        crate::shared::scaling::OrderbookDecimals {
            orderbook_id: self.orderbook_id.as_str().to_string(),
            base_decimals,
            quote_decimals,
            price_decimals: (6i16 + quote_decimals as i16 - base_decimals as i16).max(0) as u8,
            tick_size: self.tick_size.max(0) as u64,
        }
    }

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

impl OrderBookPair {
    #[cfg(test)]
    pub fn test_new(
        orderbook_id: impl Into<String>,
        base_decimals: u16,
        quote_decimals: u16,
        tick_size: i64,
    ) -> Self {
        use chrono::Utc;
        let mut base = tokens::ConditionalToken::test_new("base_mint", 0);
        let mut quote = tokens::ConditionalToken::test_new("quote_mint", 1);
        // Override decimals via serde round-trip (fields are private)
        let mut base_val = serde_json::to_value(&base).unwrap();
        base_val["decimals"] = serde_json::json!(base_decimals);
        base = serde_json::from_value(base_val).unwrap();
        let mut quote_val = serde_json::to_value(&quote).unwrap();
        quote_val["decimals"] = serde_json::json!(quote_decimals);
        quote = serde_json::from_value(quote_val).unwrap();

        Self {
            id: 1,
            market_pubkey: PubkeyStr::from("market"),
            orderbook_id: OrderBookId::from(orderbook_id.into()),
            base,
            quote,
            outcome_index: 0,
            tick_size,
            total_bids: 0,
            total_asks: 0,
            last_trade_price: None,
            last_trade_time: None,
            active: true,
        }
    }
}

