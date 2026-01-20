//! Price utilities for the Lightcone SDK.
//!
//! This module provides constants and functions for working with scaled prices.
//! All prices in the Lightcone protocol are represented as u64 values scaled by 1e6.

/// Price scaling factor (1e6).
///
/// All prices are stored as integers scaled by this factor.
/// For example, a price of 0.5 is stored as 500,000.
pub const PRICE_SCALE: u64 = 1_000_000;

/// Convert a scaled price (u64) to a decimal value (f64).
///
/// # Example
///
/// ```
/// use lightcone_pinocchio_sdk::shared::price::scaled_to_decimal;
///
/// assert_eq!(scaled_to_decimal(500_000), 0.5);
/// assert_eq!(scaled_to_decimal(1_000_000), 1.0);
/// ```
pub fn scaled_to_decimal(scaled: u64) -> f64 {
    scaled as f64 / PRICE_SCALE as f64
}

/// Convert a decimal value (f64) to a scaled price (u64).
///
/// # Example
///
/// ```
/// use lightcone_pinocchio_sdk::shared::price::decimal_to_scaled;
///
/// assert_eq!(decimal_to_scaled(0.5), 500_000);
/// assert_eq!(decimal_to_scaled(1.0), 1_000_000);
/// ```
pub fn decimal_to_scaled(decimal: f64) -> u64 {
    (decimal * PRICE_SCALE as f64) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_scaling() {
        assert_eq!(scaled_to_decimal(500_000), 0.5);
        assert_eq!(scaled_to_decimal(1_000_000), 1.0);
        assert_eq!(scaled_to_decimal(0), 0.0);
        assert_eq!(scaled_to_decimal(123_456), 0.123456);
    }

    #[test]
    fn test_decimal_to_scaled() {
        assert_eq!(decimal_to_scaled(0.5), 500_000);
        assert_eq!(decimal_to_scaled(1.0), 1_000_000);
        assert_eq!(decimal_to_scaled(0.0), 0);
        assert_eq!(decimal_to_scaled(0.123456), 123_456);
    }

    #[test]
    fn test_roundtrip() {
        let original = 750_000u64;
        let decimal = scaled_to_decimal(original);
        let back = decimal_to_scaled(decimal);
        assert_eq!(original, back);
    }
}
