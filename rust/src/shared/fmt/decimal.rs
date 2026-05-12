//! Decimal formatting utilities for human-readable display.
//!
//! Handles `rust_decimal::Decimal` values with automatic decimal-place detection,
//! abbreviated suffixes (K/M/B/T), and conversion to on-chain base units.

use rust_decimal::prelude::*;
use std::sync::OnceLock;

static TRILLION: OnceLock<Decimal> = OnceLock::new();
static BILLION: OnceLock<Decimal> = OnceLock::new();
static MILLION: OnceLock<Decimal> = OnceLock::new();
static THOUSAND: OnceLock<Decimal> = OnceLock::new();

fn get_trillion() -> &'static Decimal {
    TRILLION.get_or_init(|| Decimal::from_str("1000000000000").unwrap())
}

fn get_billion() -> &'static Decimal {
    BILLION.get_or_init(|| Decimal::from_str("1000000000").unwrap())
}

fn get_million() -> &'static Decimal {
    MILLION.get_or_init(|| Decimal::from_str("1000000").unwrap())
}

fn get_thousand() -> &'static Decimal {
    THOUSAND.get_or_init(|| Decimal::from_str("1000").unwrap())
}

fn display_decimals(abs_value: &Decimal) -> usize {
    super::constants::display_decimals_by(|threshold| {
        let threshold =
            Decimal::from_str(threshold).expect("display decimal threshold must parse as Decimal");
        abs_value >= &threshold
    })
}

/// Format a `Decimal` for display with magnitude-based decimal places.
pub fn display(value: &Decimal) -> String {
    let decimals = display_decimals(&value.abs());
    let rounded =
        value.round_dp_with_strategy(decimals as u32, RoundingStrategy::MidpointAwayFromZero);
    super::num::display_default_formatted_string(format!("{:.1$}", rounded, decimals))
}

/// Abbreviate a `Decimal` with k/m/b/t suffixes.
pub fn abbr_number(amount: &Decimal, digits: Option<usize>, show_sign: Option<bool>) -> String {
    let digits = digits.unwrap_or(2);
    let show_sign = show_sign.unwrap_or(true);
    let sign = if show_sign && amount < &Decimal::ZERO {
        "-"
    } else {
        ""
    };
    let abs_amount = amount.abs();

    if abs_amount >= *get_trillion() {
        format!(
            "{}{}t",
            sign,
            format!(
                "{:.precision$}",
                abs_amount / get_trillion(),
                precision = digits
            )
        )
    } else if abs_amount >= *get_billion() {
        format!(
            "{}{}b",
            sign,
            format!(
                "{:.precision$}",
                abs_amount / get_billion(),
                precision = digits
            )
        )
    } else if abs_amount >= *get_million() {
        format!(
            "{}{}m",
            sign,
            format!(
                "{:.precision$}",
                abs_amount / get_million(),
                precision = digits
            )
        )
    } else if abs_amount >= *get_thousand() {
        format!(
            "{}{}k",
            sign,
            format!(
                "{:.precision$}",
                abs_amount / get_thousand(),
                precision = digits
            )
        )
    } else {
        format!(
            "{}{}",
            sign,
            format!("{:.precision$}", abs_amount, precision = digits)
        )
    }
}

/// Converts a human-readable `Decimal` to token base units (u64).
///
/// Scales the decimal value by `10^decimals` and converts to u64.
/// For example, 10.5 USDC (6 decimals) becomes 10_500_000u64.
///
/// Returns `None` if the scaled value cannot be represented as a u64
/// (e.g., overflow or negative value).
pub fn to_base_units(value: &Decimal, decimals: u16) -> Option<u64> {
    let scale = Decimal::from(10u64.pow(decimals as u32));
    (value * scale).to_u64()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_display_zero() {
        assert_eq!(display(&Decimal::ZERO), "0");
    }

    #[test]
    fn test_display_tiered_decimals() {
        assert_eq!(display(&dec("12345.67")), "12,346");
        assert_eq!(display(&dec("1234.56")), "1,234.6");
        assert_eq!(display(&dec("123.456")), "123.46");
        assert_eq!(display(&dec("15.4567")), "15.457");
        assert_eq!(display(&dec("1.23456")), "1.2346");
        assert_eq!(display(&dec("0.123456")), "0.1235");
        assert_eq!(display(&dec("0.012345")), "0.01235");
    }

    #[test]
    fn test_display_tier_boundaries() {
        assert_eq!(display(&dec("10000")), "10,000");
        assert_eq!(display(&dec("9999.99")), "10,000.0");
        assert_eq!(display(&dec("1000")), "1,000.0");
        assert_eq!(display(&dec("999.999")), "1,000.00");
        assert_eq!(display(&dec("100")), "100.00");
        assert_eq!(display(&dec("99.9999")), "100.000");
        assert_eq!(display(&dec("10")), "10.000");
        assert_eq!(display(&dec("9.87654")), "9.8765");
        assert_eq!(display(&dec("1")), "1.0000");
        assert_eq!(display(&dec("0.999999")), "1.0000");
        assert_eq!(display(&dec("0.1")), "0.1000");
        assert_eq!(display(&dec("0.099999")), "0.10000");
    }

    #[test]
    fn test_display_small_values_cap_at_five_decimals() {
        assert_eq!(display(&dec("0.01")), "0.01000");
        assert_eq!(display(&dec("0.00003")), "0.00003");
        assert_eq!(display(&dec("0.000004")), "0");
        assert_eq!(display(&dec("0.000000001")), "0");
    }

    #[test]
    fn test_display_negative_values() {
        assert_eq!(display(&dec("-1234.56")), "-1,234.6");
        assert_eq!(display(&dec("-15.4567")), "-15.457");
        assert_eq!(display(&dec("-0.00003")), "-0.00003");
        assert_eq!(display(&dec("-0.000004")), "0");
    }

    #[test]
    fn test_to_base_units_usdc_6_decimals() {
        assert_eq!(to_base_units(&dec("0"), 6), Some(0));
        assert_eq!(to_base_units(&dec("1"), 6), Some(1_000_000));
        assert_eq!(to_base_units(&dec("10.5"), 6), Some(10_500_000));
        assert_eq!(to_base_units(&dec("0.000001"), 6), Some(1));
    }

    #[test]
    fn test_to_base_units_negative_values() {
        assert_eq!(to_base_units(&dec("-1"), 6), None);
    }

    #[test]
    fn test_abbr_number_below_thousand() {
        assert_eq!(abbr_number(&dec("0"), None, None), "0.00");
        assert_eq!(abbr_number(&dec("1"), None, None), "1.00");
        assert_eq!(abbr_number(&dec("999"), None, None), "999.00");
    }

    #[test]
    fn test_abbr_number_thousands() {
        assert_eq!(abbr_number(&dec("1000"), None, None), "1.00k");
        assert_eq!(abbr_number(&dec("1500"), None, None), "1.50k");
        assert_eq!(abbr_number(&dec("12345"), None, None), "12.34k");
    }

    #[test]
    fn test_abbr_number_millions() {
        assert_eq!(abbr_number(&dec("1000000"), None, None), "1.00m");
        assert_eq!(abbr_number(&dec("1500000"), None, None), "1.50m");
    }

    #[test]
    fn test_abbr_number_negative() {
        assert_eq!(abbr_number(&dec("-1500000"), None, None), "-1.50m");
        assert_eq!(abbr_number(&dec("-1500000"), None, Some(false)), "1.50m");
    }
}
