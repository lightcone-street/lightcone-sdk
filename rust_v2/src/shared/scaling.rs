//! Pure conversion module for price/size to raw lamport amounts.
//!
//! All math uses `rust_decimal::Decimal` for exact integer arithmetic.
//! No async, no network calls.

use std::fmt;

use rust_decimal::prelude::*;
use rust_decimal::Decimal;

use crate::program::types::OrderSide;

/// Decimal metadata for an orderbook (cached permanently).
#[derive(Debug, Clone)]
pub struct OrderbookDecimals {
    pub orderbook_id: String,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub price_decimals: u8,
}

/// Result of converting price + size to raw u64 amounts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaledAmounts {
    pub amount_in: u64,
    pub amount_out: u64,
}

/// Errors that can occur during price/size scaling.
#[derive(Debug, Clone)]
pub enum ScalingError {
    NonPositivePrice(String),
    NonPositiveSize(String),
    Overflow { context: String },
    ZeroAmount,
    FractionalAmount { value: String },
    InvalidDecimal { input: String, reason: String },
}

impl fmt::Display for ScalingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScalingError::NonPositivePrice(v) => write!(f, "Price must be positive, got {}", v),
            ScalingError::NonPositiveSize(v) => write!(f, "Size must be positive, got {}", v),
            ScalingError::Overflow { context } => write!(f, "Overflow: {}", context),
            ScalingError::ZeroAmount => write!(f, "Computed amount is zero"),
            ScalingError::FractionalAmount { value } => {
                write!(f, "Fractional lamports not allowed: {}", value)
            }
            ScalingError::InvalidDecimal { input, reason } => {
                write!(f, "Invalid decimal '{}': {}", input, reason)
            }
        }
    }
}

impl std::error::Error for ScalingError {}

/// Convert human-readable price and size into raw u64 maker/taker amounts.
///
/// # Conversion math
///
/// ```text
/// base_lamports  = size  * 10^base_decimals
/// quote_lamports = price * size * 10^quote_decimals
/// ```
///
/// Then assign based on side:
///
/// | Side | amount_in (gives) | amount_out (receives) |
/// |------|-------------------|----------------------|
/// | BID  | quote_lamports    | base_lamports        |
/// | ASK  | base_lamports     | quote_lamports       |
pub fn scale_price_size(
    price: Decimal,
    size: Decimal,
    side: OrderSide,
    decimals: &OrderbookDecimals,
) -> Result<ScaledAmounts, ScalingError> {
    // 1. Validate inputs
    if price <= Decimal::ZERO {
        return Err(ScalingError::NonPositivePrice(price.to_string()));
    }
    if size <= Decimal::ZERO {
        return Err(ScalingError::NonPositiveSize(size.to_string()));
    }

    // 2. Compute lamport amounts
    let base_multiplier = Decimal::from(
        10u64
            .checked_pow(decimals.base_decimals as u32)
            .ok_or_else(|| ScalingError::Overflow {
                context: format!("10^{} overflow", decimals.base_decimals),
            })?,
    );

    let quote_multiplier = Decimal::from(
        10u64
            .checked_pow(decimals.quote_decimals as u32)
            .ok_or_else(|| ScalingError::Overflow {
                context: format!("10^{} overflow", decimals.quote_decimals),
            })?,
    );

    let base_lamports = size
        .checked_mul(base_multiplier)
        .ok_or_else(|| ScalingError::Overflow {
            context: "size * 10^base_decimals".to_string(),
        })?;

    let quote_lamports = price
        .checked_mul(size)
        .ok_or_else(|| ScalingError::Overflow {
            context: "price * size".to_string(),
        })?
        .checked_mul(quote_multiplier)
        .ok_or_else(|| ScalingError::Overflow {
            context: "price * size * 10^quote_decimals".to_string(),
        })?;

    // 3. Validate whole numbers (no fractional lamports)
    if base_lamports.fract() != Decimal::ZERO {
        return Err(ScalingError::FractionalAmount {
            value: format!("base_lamports = {}", base_lamports),
        });
    }
    if quote_lamports.fract() != Decimal::ZERO {
        return Err(ScalingError::FractionalAmount {
            value: format!("quote_lamports = {}", quote_lamports),
        });
    }

    // 4. Convert to u64
    let base_u64 = base_lamports
        .to_u64()
        .ok_or_else(|| ScalingError::Overflow {
            context: format!("base_lamports {} does not fit in u64", base_lamports),
        })?;

    let quote_u64 = quote_lamports
        .to_u64()
        .ok_or_else(|| ScalingError::Overflow {
            context: format!("quote_lamports {} does not fit in u64", quote_lamports),
        })?;

    // 5. Validate non-zero
    if base_u64 == 0 || quote_u64 == 0 {
        return Err(ScalingError::ZeroAmount);
    }

    // 6. Assign based on side
    let (amount_in, amount_out) = match side {
        OrderSide::Bid => (quote_u64, base_u64),
        OrderSide::Ask => (base_u64, quote_u64),
    };

    Ok(ScaledAmounts {
        amount_in,
        amount_out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimals_6_6() -> OrderbookDecimals {
        OrderbookDecimals {
            orderbook_id: "test".to_string(),
            base_decimals: 6,
            quote_decimals: 6,
            price_decimals: 2,
        }
    }

    fn decimals_6_9() -> OrderbookDecimals {
        OrderbookDecimals {
            orderbook_id: "test".to_string(),
            base_decimals: 6,
            quote_decimals: 9,
            price_decimals: 2,
        }
    }

    #[test]
    fn test_bid_basic() {
        // BID: price=0.65, size=100, decimals=6/6
        // base_lamports  = 100 * 10^6 = 100_000_000
        // quote_lamports = 0.65 * 100 * 10^6 = 65_000_000
        // BID: maker gives quote, taker gives base
        let result = scale_price_size(
            Decimal::from_str("0.65").unwrap(),
            Decimal::from_str("100").unwrap(),
            OrderSide::Bid,
            &decimals_6_6(),
        )
        .unwrap();

        assert_eq!(result.amount_in, 65_000_000);
        assert_eq!(result.amount_out, 100_000_000);
    }

    #[test]
    fn test_ask_basic() {
        // ASK: price=0.65, size=100, decimals=6/6
        // base_lamports  = 100 * 10^6 = 100_000_000
        // quote_lamports = 0.65 * 100 * 10^6 = 65_000_000
        // ASK: maker gives base, taker gives quote
        let result = scale_price_size(
            Decimal::from_str("0.65").unwrap(),
            Decimal::from_str("100").unwrap(),
            OrderSide::Ask,
            &decimals_6_6(),
        )
        .unwrap();

        assert_eq!(result.amount_in, 100_000_000);
        assert_eq!(result.amount_out, 65_000_000);
    }

    #[test]
    fn test_different_decimals() {
        // base=6, quote=9
        // base_lamports  = 100 * 10^6 = 100_000_000
        // quote_lamports = 0.65 * 100 * 10^9 = 65_000_000_000
        let result = scale_price_size(
            Decimal::from_str("0.65").unwrap(),
            Decimal::from_str("100").unwrap(),
            OrderSide::Bid,
            &decimals_6_9(),
        )
        .unwrap();

        assert_eq!(result.amount_in, 65_000_000_000);
        assert_eq!(result.amount_out, 100_000_000);
    }

    #[test]
    fn test_zero_price_rejected() {
        let result = scale_price_size(
            Decimal::ZERO,
            Decimal::from_str("100").unwrap(),
            OrderSide::Bid,
            &decimals_6_6(),
        );
        assert!(matches!(result, Err(ScalingError::NonPositivePrice(_))));
    }

    #[test]
    fn test_negative_price_rejected() {
        let result = scale_price_size(
            Decimal::from_str("-0.5").unwrap(),
            Decimal::from_str("100").unwrap(),
            OrderSide::Bid,
            &decimals_6_6(),
        );
        assert!(matches!(result, Err(ScalingError::NonPositivePrice(_))));
    }

    #[test]
    fn test_zero_size_rejected() {
        let result = scale_price_size(
            Decimal::from_str("0.65").unwrap(),
            Decimal::ZERO,
            OrderSide::Bid,
            &decimals_6_6(),
        );
        assert!(matches!(result, Err(ScalingError::NonPositiveSize(_))));
    }

    #[test]
    fn test_negative_size_rejected() {
        let result = scale_price_size(
            Decimal::from_str("0.65").unwrap(),
            Decimal::from_str("-10").unwrap(),
            OrderSide::Bid,
            &decimals_6_6(),
        );
        assert!(matches!(result, Err(ScalingError::NonPositiveSize(_))));
    }

    #[test]
    fn test_fractional_lamports_rejected() {
        // size=0.0000001 with 6 decimals = 0.1 lamports (fractional)
        let result = scale_price_size(
            Decimal::from_str("1").unwrap(),
            Decimal::from_str("0.0000001").unwrap(),
            OrderSide::Bid,
            &decimals_6_6(),
        );
        assert!(matches!(result, Err(ScalingError::FractionalAmount { .. })));
    }

    #[test]
    fn test_overflow_u64_rejected() {
        // Huge size that overflows u64
        let result = scale_price_size(
            Decimal::from_str("1").unwrap(),
            Decimal::from_str("99999999999999999999").unwrap(),
            OrderSide::Bid,
            &decimals_6_6(),
        );
        assert!(matches!(result, Err(ScalingError::Overflow { .. })));
    }

    #[test]
    fn test_small_valid_amounts() {
        // Minimum valid: 1 lamport each
        // size = 0.000001 (1 lamport with 6 decimals)
        // price = 1.0 -> quote = 0.000001 * 10^6 = 1
        let result = scale_price_size(
            Decimal::from_str("1").unwrap(),
            Decimal::from_str("0.000001").unwrap(),
            OrderSide::Bid,
            &decimals_6_6(),
        )
        .unwrap();

        assert_eq!(result.amount_in, 1); // quote
        assert_eq!(result.amount_out, 1); // base
    }

    #[test]
    fn test_whole_number_price_and_size() {
        // price=2, size=50, decimals=6/6
        // base = 50 * 10^6 = 50_000_000
        // quote = 2 * 50 * 10^6 = 100_000_000
        let result = scale_price_size(
            Decimal::from_str("2").unwrap(),
            Decimal::from_str("50").unwrap(),
            OrderSide::Ask,
            &decimals_6_6(),
        )
        .unwrap();

        assert_eq!(result.amount_in, 50_000_000);
        assert_eq!(result.amount_out, 100_000_000);
    }
}
