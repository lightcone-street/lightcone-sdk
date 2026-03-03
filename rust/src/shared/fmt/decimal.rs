//! Decimal formatting utilities for human-readable display.
//!
//! Handles `rust_decimal::Decimal` values with subscript notation for very small
//! numbers, automatic decimal-place detection, abbreviated suffixes (K/M/B/T),
//! and conversion to on-chain base units.

use rust_decimal::prelude::*;
use std::sync::OnceLock;

static HUNDRED: OnceLock<Decimal> = OnceLock::new();
static TRILLION: OnceLock<Decimal> = OnceLock::new();
static BILLION: OnceLock<Decimal> = OnceLock::new();
static MILLION: OnceLock<Decimal> = OnceLock::new();
static THOUSAND: OnceLock<Decimal> = OnceLock::new();

fn get_hundred() -> &'static Decimal {
    HUNDRED.get_or_init(|| Decimal::from(100))
}

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

enum DecimalFormat {
    Standard(u32),
    Subscript { zeros: u32, significant: String },
}

#[inline]
fn count_digits_u128(n: u128) -> u32 {
    if n == 0 {
        return 1;
    }
    n.ilog10() + 1
}

#[inline]
fn extract_significant_digits(mantissa: u128, total_digits: u32, want_digits: u32) -> u128 {
    if total_digits <= want_digits {
        return mantissa;
    }
    let divisor = 10u128.pow(total_digits - want_digits);
    mantissa / divisor
}

fn get_decimal_format(value: &Decimal) -> DecimalFormat {
    if value.is_zero() {
        return DecimalFormat::Standard(2);
    }

    let abs_value = value.abs();

    if abs_value >= *get_hundred() {
        return DecimalFormat::Standard(0);
    }

    if abs_value >= Decimal::ONE {
        return DecimalFormat::Standard(2);
    }

    let scale = abs_value.scale();
    let mantissa = abs_value.mantissa().unsigned_abs();

    let mantissa_digits = count_digits_u128(mantissa);
    let leading_zeros = (scale as u32).saturating_sub(mantissa_digits);

    if leading_zeros > 5 {
        let sig_digits = mantissa_digits.min(4);
        let significant = extract_significant_digits(mantissa, mantissa_digits, sig_digits);

        let mut sig = significant;
        while sig > 0 && sig % 10 == 0 {
            sig /= 10;
        }

        DecimalFormat::Subscript {
            zeros: leading_zeros,
            significant: if sig == 0 {
                "0".to_string()
            } else {
                sig.to_string()
            },
        }
    } else {
        DecimalFormat::Standard((leading_zeros + 3).min(8))
    }
}

/// Format a `Decimal` for display, handling subscript notation for very small values.
pub fn display(value: &Decimal) -> String {
    match get_decimal_format(value) {
        DecimalFormat::Standard(decimals) => {
            let rounded = value.round_dp(decimals);
            super::num::display_formatted_string(rounded.to_string())
        }
        DecimalFormat::Subscript { zeros, significant } => {
            if value.is_sign_negative() {
                format!("-0.0({}){}", zeros, significant)
            } else {
                format!("0.0({}){}", zeros, significant)
            }
        }
    }
}

/// Abbreviate a `Decimal` with K/M/B/T suffixes.
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
            "{}{}T",
            sign,
            format!(
                "{:.precision$}",
                abs_amount / get_trillion(),
                precision = digits
            )
        )
    } else if abs_amount >= *get_billion() {
        format!(
            "{}{}B",
            sign,
            format!(
                "{:.precision$}",
                abs_amount / get_billion(),
                precision = digits
            )
        )
    } else if abs_amount >= *get_million() {
        format!(
            "{}{}M",
            sign,
            format!(
                "{:.precision$}",
                abs_amount / get_million(),
                precision = digits
            )
        )
    } else if abs_amount >= *get_thousand() {
        format!(
            "{}{}K",
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
    fn test_display_large_values_no_decimals() {
        assert_eq!(display(&dec("100")), "100");
        assert_eq!(display(&dec("100.99")), "101");
        assert_eq!(display(&dec("1234.56")), "1,235");
        assert_eq!(display(&dec("999999.999")), "1,000,000");
    }

    #[test]
    fn test_display_medium_values_two_decimals() {
        assert_eq!(display(&dec("1.00")), "1");
        assert_eq!(display(&dec("1.50")), "1.5");
        assert_eq!(display(&dec("1.505058")), "1.51");
        assert_eq!(display(&dec("15.456")), "15.46");
        assert_eq!(display(&dec("99.999")), "100");
    }

    #[test]
    fn test_display_small_values_standard() {
        assert_eq!(display(&dec("0.1")), "0.1");
        assert_eq!(display(&dec("0.12")), "0.12");
        assert_eq!(display(&dec("0.123")), "0.123");
        assert_eq!(display(&dec("0.00123")), "0.00123");
        assert_eq!(display(&dec("0.000123")), "0.000123");
        assert_eq!(display(&dec("0.0000123")), "0.0000123");
    }

    #[test]
    fn test_display_very_small_values_subscript() {
        assert_eq!(display(&dec("0.0000001")), "0.0(6)1");
        assert_eq!(display(&dec("0.00000015")), "0.0(6)15");
        assert_eq!(display(&dec("0.000000000000000000015")), "0.0(19)15");
        assert_eq!(display(&dec("0.00000012345")), "0.0(6)1234");
    }

    #[test]
    fn test_display_negative_values() {
        assert_eq!(display(&dec("-1234.56")), "-1,235");
        assert_eq!(display(&dec("-15.456")), "-15.46");
        assert_eq!(display(&dec("-0.00123")), "-0.00123");
        assert_eq!(display(&dec("-0.0000001")), "-0.0(6)1");
    }

    #[test]
    fn test_display_trailing_zeros_trimmed() {
        assert_eq!(display(&dec("0.00000010")), "0.0(6)1");
        assert_eq!(display(&dec("0.000000100")), "0.0(6)1");
        assert_eq!(display(&dec("0.0000001200")), "0.0(6)12");
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
        assert_eq!(abbr_number(&dec("1000"), None, None), "1.00K");
        assert_eq!(abbr_number(&dec("1500"), None, None), "1.50K");
        assert_eq!(abbr_number(&dec("12345"), None, None), "12.34K");
    }

    #[test]
    fn test_abbr_number_millions() {
        assert_eq!(abbr_number(&dec("1000000"), None, None), "1.00M");
        assert_eq!(abbr_number(&dec("1500000"), None, None), "1.50M");
    }

    #[test]
    fn test_abbr_number_negative() {
        assert_eq!(abbr_number(&dec("-1500000"), None, None), "-1.50M");
        assert_eq!(abbr_number(&dec("-1500000"), None, Some(false)), "1.50M");
    }
}
