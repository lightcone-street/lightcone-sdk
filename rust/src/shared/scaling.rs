//! Exact order construction and trading-rule validation.
//!
//! The matching engine validates the signed ratio. This module therefore uses
//! decimal-string parsing and integer arithmetic only: it never rounds, floors,
//! or aligns a caller's value implicitly.

use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::program::types::OrderSide;

pub const PRICE_SCALE: u64 = 1_000_000;
pub const I64_MAX_U64: u64 = i64::MAX as u64;
pub const NONCE_MAX: u64 = u32::MAX as u64;

/// Immutable admission rules returned by the orderbook decimals endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TradingRules {
    pub base_size_decimals: u8,
    pub max_price_decimals: u8,
    pub max_price_significant_figures: u8,
    pub integer_prices_always_allowed: bool,
    /// Display-only formatted quantum.
    pub price_quantum: String,
    #[serde(with = "biguint_string")]
    pub price_quantum_raw: BigUint,
    /// Display-only formatted quantum.
    pub base_size_quantum: String,
    #[serde(with = "biguint_string")]
    pub base_size_quantum_raw: BigUint,
}

/// Complete rules required to construct or preflight an order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderbookRules {
    pub orderbook_id: String,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub price_decimals: u8,
    pub trading_rules: TradingRules,
}

impl OrderbookRules {
    /// Ensure these rules belong to the orderbook being constructed or
    /// submitted.
    pub fn validate_for_orderbook(&self, orderbook_id: &str) -> Result<(), ScalingError> {
        if self.orderbook_id == orderbook_id {
            Ok(())
        } else {
            Err(ScalingError::OrderbookMismatch {
                expected: orderbook_id.to_string(),
                actual: self.orderbook_id.clone(),
            })
        }
    }
}

mod biguint_string {
    use super::*;

    pub fn serialize<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_str_radix(10))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        BigUint::parse_bytes(value.as_bytes(), 10)
            .ok_or_else(|| serde::de::Error::custom("expected an unsigned decimal integer string"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaledAmounts {
    pub amount_in: u64,
    pub amount_out: u64,
    pub price_raw: u64,
    pub base_atoms: u64,
    pub quote_atoms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScalingError {
    #[error("invalid decimal '{input}': {reason}")]
    InvalidDecimal { input: String, reason: String },
    #[error("price must be positive, got {0}")]
    NonPositivePrice(String),
    #[error("size must be positive, got {0}")]
    NonPositiveSize(String),
    #[error("INVALID_SIZE_DECIMALS")]
    InvalidSizeDecimals,
    #[error("INVALID_PRICE_DECIMALS")]
    InvalidPriceDecimals,
    #[error("INVALID_PRICE_SIGNIFICANT_FIGURES")]
    InvalidPriceSignificantFigures,
    #[error("PRICE_NOT_EXACTLY_REPRESENTABLE")]
    PriceNotExactlyRepresentable,
    #[error("PRICE_OUT_OF_RANGE")]
    PriceOutOfRange,
    #[error("ORDER_FIELD_OUT_OF_RANGE: {field}")]
    OrderFieldOutOfRange { field: &'static str },
    #[error("TRIGGER_PRICE_OUT_OF_RANGE")]
    TriggerPriceOutOfRange,
    #[error("trading rules for orderbook '{actual}' cannot be used for '{expected}'")]
    OrderbookMismatch { expected: String, actual: String },
}

/// Convert a decimal string to an integer at `decimals` without rounding.
/// Exponent notation is accepted. Fractional trailing zeros beyond the scale
/// are accepted because they do not change the represented value.
pub fn exact_scaled_integer(value: &str, decimals: u8) -> Result<BigUint, ScalingError> {
    let input = value.trim();
    if input.is_empty() {
        return invalid_decimal(value, "empty input");
    }
    if input.starts_with('-') {
        return invalid_decimal(value, "negative values are not supported");
    }
    let unsigned = input.strip_prefix('+').unwrap_or(input);
    let mut exponent_parts = unsigned.split(['e', 'E']);
    let mantissa = exponent_parts.next().unwrap_or_default();
    let exponent = match exponent_parts.next() {
        Some(raw) => raw
            .parse::<i32>()
            .map_err(|_| ScalingError::InvalidDecimal {
                input: value.to_string(),
                reason: "invalid exponent".to_string(),
            })?,
        None => 0,
    };
    if exponent_parts.next().is_some() || exponent.unsigned_abs() > 10_000 {
        return invalid_decimal(value, "invalid or unsupported exponent");
    }

    let mut decimal_parts = mantissa.split('.');
    let whole = decimal_parts.next().unwrap_or_default();
    let fraction = decimal_parts.next().unwrap_or_default();
    if decimal_parts.next().is_some()
        || (whole.is_empty() && fraction.is_empty())
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return invalid_decimal(value, "expected base-10 decimal syntax");
    }

    let coefficient = format!("{whole}{fraction}");
    let trimmed = coefficient.trim_start_matches('0');
    if trimmed.is_empty() {
        return Ok(BigUint::zero());
    }

    let shift = i64::from(decimals) + i64::from(exponent) - fraction.len() as i64;
    let digits = if shift >= 0 {
        let zero_count = usize::try_from(shift).map_err(|_| ScalingError::InvalidDecimal {
            input: value.to_string(),
            reason: "scaled value is too large".to_string(),
        })?;
        if trimmed.len().saturating_add(zero_count) > 10_000 {
            return invalid_decimal(value, "scaled value is too large");
        }
        format!("{trimmed}{}", "0".repeat(zero_count))
    } else {
        let remove = usize::try_from(-shift).map_err(|_| ScalingError::InvalidDecimal {
            input: value.to_string(),
            reason: "scaled value is too precise".to_string(),
        })?;
        if remove > trimmed.len() {
            return invalid_decimal(value, "cannot be represented exactly at this scale");
        }
        let split = trimmed.len() - remove;
        if !trimmed.as_bytes()[split..].iter().all(|b| *b == b'0') {
            return invalid_decimal(value, "cannot be represented exactly at this scale");
        }
        let remaining = &trimmed[..split];
        if remaining.is_empty() {
            "0".to_string()
        } else {
            remaining.to_string()
        }
    };

    BigUint::parse_bytes(digits.as_bytes(), 10).ok_or_else(|| ScalingError::InvalidDecimal {
        input: value.to_string(),
        reason: "invalid integer coefficient".to_string(),
    })
}

fn invalid_decimal<T>(input: &str, reason: &str) -> Result<T, ScalingError> {
    Err(ScalingError::InvalidDecimal {
        input: input.to_string(),
        reason: reason.to_string(),
    })
}

fn checked_i64(value: &BigUint, field: &'static str) -> Result<u64, ScalingError> {
    let parsed = value
        .to_u64()
        .filter(|v| *v <= I64_MAX_U64)
        .ok_or(ScalingError::OrderFieldOutOfRange { field })?;
    Ok(parsed)
}

fn significant_digits(mut value: BigUint) -> usize {
    let ten = BigUint::from(10u8);
    while !value.is_zero() && (&value % &ten).is_zero() {
        value /= &ten;
    }
    value.to_str_radix(10).len()
}

fn validate_price_raw(price_raw: &BigUint, rules: &OrderbookRules) -> Result<u64, ScalingError> {
    if price_raw.is_zero() || price_raw > &BigUint::from(I64_MAX_U64) {
        return Err(ScalingError::PriceOutOfRange);
    }
    let human_scale = BigUint::from(10u8).pow(u32::from(rules.price_decimals));
    let is_integer = (price_raw % &human_scale).is_zero();
    if !is_integer {
        if rules.trading_rules.price_quantum_raw.is_zero()
            || (price_raw % &rules.trading_rules.price_quantum_raw) != BigUint::zero()
        {
            return Err(ScalingError::InvalidPriceDecimals);
        }
        if significant_digits(price_raw.clone())
            > usize::from(rules.trading_rules.max_price_significant_figures)
        {
            return Err(ScalingError::InvalidPriceSignificantFigures);
        }
    }
    price_raw.to_u64().ok_or(ScalingError::PriceOutOfRange)
}

fn validate_base_atoms(base_atoms: &BigUint, rules: &OrderbookRules) -> Result<u64, ScalingError> {
    if base_atoms.is_zero() {
        return Err(ScalingError::NonPositiveSize("0".to_string()));
    }
    if rules.trading_rules.base_size_quantum_raw.is_zero()
        || (base_atoms % &rules.trading_rules.base_size_quantum_raw) != BigUint::zero()
    {
        return Err(ScalingError::InvalidSizeDecimals);
    }
    checked_i64(base_atoms, "base amount")
}

/// Construct signed amounts from an exact human price and base size.
pub fn scale_price_size(
    price: &str,
    size: &str,
    side: OrderSide,
    rules: &OrderbookRules,
) -> Result<ScaledAmounts, ScalingError> {
    let price_raw_big = exact_scaled_integer(price, rules.price_decimals)?;
    if price_raw_big.is_zero() {
        return Err(ScalingError::NonPositivePrice(price.to_string()));
    }
    let base_atoms_big = exact_scaled_integer(size, rules.base_decimals)?;
    if base_atoms_big.is_zero() {
        return Err(ScalingError::NonPositiveSize(size.to_string()));
    }

    let price_raw = validate_price_raw(&price_raw_big, rules)?;
    let base_atoms = validate_base_atoms(&base_atoms_big, rules)?;
    let numerator = &price_raw_big * &base_atoms_big;
    let scale = BigUint::from(PRICE_SCALE);
    if (&numerator % &scale) != BigUint::zero() {
        return Err(ScalingError::PriceNotExactlyRepresentable);
    }
    let quote_atoms_big = numerator / scale;
    if quote_atoms_big.is_zero() {
        return Err(ScalingError::OrderFieldOutOfRange {
            field: "quote amount",
        });
    }
    let quote_atoms = checked_i64(&quote_atoms_big, "quote amount")?;
    let (amount_in, amount_out) = match side {
        OrderSide::Bid => (quote_atoms, base_atoms),
        OrderSide::Ask => (base_atoms, quote_atoms),
    };
    Ok(ScaledAmounts {
        amount_in,
        amount_out,
        price_raw,
        base_atoms,
        quote_atoms,
    })
}

/// Preflight caller-supplied signed amounts against the same engine rules.
pub fn validate_raw_amounts(
    amount_in: u64,
    amount_out: u64,
    side: OrderSide,
    rules: &OrderbookRules,
) -> Result<ScaledAmounts, ScalingError> {
    if amount_in == 0 || amount_in > I64_MAX_U64 {
        return Err(ScalingError::OrderFieldOutOfRange { field: "amount_in" });
    }
    if amount_out == 0 || amount_out > I64_MAX_U64 {
        return Err(ScalingError::OrderFieldOutOfRange {
            field: "amount_out",
        });
    }
    let (base_atoms, quote_atoms) = match side {
        OrderSide::Bid => (amount_out, amount_in),
        OrderSide::Ask => (amount_in, amount_out),
    };
    let base_big = BigUint::from(base_atoms);
    validate_base_atoms(&base_big, rules)?;
    let numerator = BigUint::from(quote_atoms) * BigUint::from(PRICE_SCALE);
    if (&numerator % &base_big) != BigUint::zero() {
        return Err(ScalingError::PriceNotExactlyRepresentable);
    }
    let price_big = numerator / base_big;
    let price_raw = validate_price_raw(&price_big, rules)?;
    Ok(ScaledAmounts {
        amount_in,
        amount_out,
        price_raw,
        base_atoms,
        quote_atoms,
    })
}

pub fn validate_signed_fields(
    amount_in: u64,
    amount_out: u64,
    salt: u64,
    nonce: u64,
) -> Result<(), ScalingError> {
    for (field, value, zero_allowed) in [
        ("amount_in", amount_in, false),
        ("amount_out", amount_out, false),
        ("salt", salt, true),
    ] {
        if value > I64_MAX_U64 || (!zero_allowed && value == 0) {
            return Err(ScalingError::OrderFieldOutOfRange { field });
        }
    }
    if nonce > NONCE_MAX {
        return Err(ScalingError::OrderFieldOutOfRange { field: "nonce" });
    }
    Ok(())
}

pub fn validate_trigger_price(value: &str, price_decimals: u8) -> Result<u64, ScalingError> {
    let raw = exact_scaled_integer(value, price_decimals)
        .map_err(|_| ScalingError::TriggerPriceOutOfRange)?;
    if raw.is_zero() || raw > BigUint::from(I64_MAX_U64) {
        return Err(ScalingError::TriggerPriceOutOfRange);
    }
    raw.to_u64().ok_or(ScalingError::TriggerPriceOutOfRange)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> OrderbookRules {
        OrderbookRules {
            orderbook_id: "test".into(),
            base_decimals: 8,
            quote_decimals: 6,
            price_decimals: 4,
            trading_rules: TradingRules {
                base_size_decimals: 5,
                max_price_decimals: 1,
                max_price_significant_figures: 5,
                integer_prices_always_allowed: true,
                price_quantum: "0.1000".into(),
                price_quantum_raw: BigUint::from(1000u32),
                base_size_quantum: "0.00001000".into(),
                base_size_quantum_raw: BigUint::from(1000u32),
            },
        }
    }

    #[test]
    fn worked_example_bid_and_ask() {
        let bid = scale_price_size("12.3", "1.23456", OrderSide::Bid, &rules()).unwrap();
        assert_eq!((bid.amount_in, bid.amount_out), (15_185_088, 123_456_000));
        let ask = scale_price_size("12.3", "1.23456", OrderSide::Ask, &rules()).unwrap();
        assert_eq!((ask.amount_in, ask.amount_out), (123_456_000, 15_185_088));
    }

    #[test]
    fn rejects_invalid_price_size_and_significant_figures() {
        assert_eq!(
            scale_price_size("12.34", "1.23456", OrderSide::Bid, &rules()).unwrap_err(),
            ScalingError::InvalidPriceDecimals
        );
        assert_eq!(
            scale_price_size("12.3", "1.234567", OrderSide::Bid, &rules()).unwrap_err(),
            ScalingError::InvalidSizeDecimals
        );
        assert_eq!(
            scale_price_size("150250.1", "1", OrderSide::Bid, &rules()).unwrap_err(),
            ScalingError::InvalidPriceSignificantFigures
        );
    }

    #[test]
    fn accepts_large_integer_price_when_ratio_is_exact() {
        let value = scale_price_size("150250", "1", OrderSide::Bid, &rules()).unwrap();
        assert_eq!(value.price_raw, 1_502_500_000);
    }

    #[test]
    fn exact_parser_never_truncates() {
        assert_eq!(
            exact_scaled_integer("1.23000", 2).unwrap(),
            BigUint::from(123u8)
        );
        assert_eq!(
            exact_scaled_integer("1.23e2", 2).unwrap(),
            BigUint::from(12_300u16)
        );
        assert!(exact_scaled_integer("1.234", 2).is_err());
    }

    #[test]
    fn raw_ratio_must_be_exact() {
        assert_eq!(
            validate_raw_amounts(1, 3_000, OrderSide::Bid, &rules()).unwrap_err(),
            ScalingError::PriceNotExactlyRepresentable
        );
    }

    #[test]
    fn signed_range_is_i64() {
        assert!(
            validate_signed_fields(I64_MAX_U64, I64_MAX_U64, I64_MAX_U64, u32::MAX as u64).is_ok()
        );
        assert!(validate_signed_fields(I64_MAX_U64 + 1, 1, 0, 0).is_err());
    }

    #[test]
    fn rules_are_bound_to_their_orderbook() {
        assert!(rules().validate_for_orderbook("test").is_ok());
        assert_eq!(
            rules().validate_for_orderbook("other").unwrap_err(),
            ScalingError::OrderbookMismatch {
                expected: "other".into(),
                actual: "test".into(),
            }
        );
    }
}
