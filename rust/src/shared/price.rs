//! Price utilities for the Lightcone SDK.
//!
//! This module provides helper functions for working with decimal string prices.
//! The SDK now uses String types for price/size/balance fields to preserve
//! the exact decimal representation from the server, as different tokens
//! have different decimal places (USDC=6, SOL=9, BTC=8, etc.).

/// Parse a decimal string to f64 for calculations.
///
/// # Example
///
/// ```
/// use lightcone_sdk::shared::price::parse_decimal;
///
/// assert_eq!(parse_decimal("0.500000").unwrap(), 0.5);
/// assert_eq!(parse_decimal("1.000000").unwrap(), 1.0);
/// ```
pub fn parse_decimal(s: &str) -> Result<f64, std::num::ParseFloatError> {
    s.parse()
}

/// Format an f64 as a decimal string with specified precision.
///
/// # Example
///
/// ```
/// use lightcone_sdk::shared::price::format_decimal;
///
/// assert_eq!(format_decimal(0.5, 6), "0.500000");
/// assert_eq!(format_decimal(1.0, 6), "1.000000");
/// ```
pub fn format_decimal(value: f64, precision: usize) -> String {
    format!("{:.precision$}", value, precision = precision)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decimal() {
        assert_eq!(parse_decimal("0.500000").unwrap(), 0.5);
        assert_eq!(parse_decimal("1.000000").unwrap(), 1.0);
        assert_eq!(parse_decimal("0.0").unwrap(), 0.0);
        assert_eq!(parse_decimal("0.123456").unwrap(), 0.123456);
    }

    #[test]
    fn test_format_decimal() {
        assert_eq!(format_decimal(0.5, 6), "0.500000");
        assert_eq!(format_decimal(1.0, 6), "1.000000");
        assert_eq!(format_decimal(0.0, 6), "0.000000");
        assert_eq!(format_decimal(0.123456, 6), "0.123456");
    }

    #[test]
    fn test_roundtrip() {
        let original = "0.750000";
        let parsed = parse_decimal(original).unwrap();
        let back = format_decimal(parsed, 6);
        assert_eq!(original, back);
    }
}
