//! Number formatting utilities for human-readable display.
//!
//! Handles f64 values with automatic decimal-place detection and comma separators.
//! For `Decimal` formatting, use the `decimal` sibling module.

/// Adds thousands separators while preserving fractional digits.
pub fn display_formatted_string(formatted: String) -> String {
    let (integer, fraction) = if let Some((integer, fraction)) = formatted.split_once('.') {
        (integer, Some(fraction))
    } else {
        (formatted.as_str(), None)
    };

    let (sign, digits) = if let Some(digits) = integer.strip_prefix('-') {
        ("-", digits)
    } else {
        ("", integer)
    };

    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    let integer_part = grouped.chars().rev().collect::<String>();

    if let Some(fraction) = fraction {
        format!("{}{}.{}", sign, integer_part, fraction)
    } else {
        format!("{}{}", sign, integer_part)
    }
}

/// Format an f64 for display with auto-detected decimal places.
pub fn display(amount: &f64) -> String {
    display_with_decimals(amount, super::display_decimals_f64(amount.abs()))
}

/// Format an f64 for display with explicit decimal places.
pub fn display_with_decimals(amount: &f64, decimals: usize) -> String {
    let formatted = format!("{:.1$}", amount, decimals);
    display_formatted_string(formatted)
}

/// Convert an on-chain integer value to f64 given the token's decimal places.
///
/// e.g. `to_decimal_value(1_500_000_000, 9)` → `1.5`
pub fn to_decimal_value(value: u64, decimals: u64) -> f64 {
    value as f64 / 10f64.powi(decimals as i32)
}

/// Convert an f64 value back to on-chain integer representation.
///
/// e.g. `from_decimal_value(1.5, 9)` → `1_500_000_000`
pub fn from_decimal_value(value: f64, decimals: u64) -> u64 {
    (value * 10f64.powi(decimals as i32)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_formatted_string_integers() {
        assert_eq!(display_formatted_string("0".to_string()), "0");
        assert_eq!(display_formatted_string("1".to_string()), "1");
        assert_eq!(display_formatted_string("123".to_string()), "123");
    }

    #[test]
    fn test_display_formatted_string_thousands_separator() {
        assert_eq!(display_formatted_string("1000".to_string()), "1,000");
        assert_eq!(display_formatted_string("12345".to_string()), "12,345");
        assert_eq!(display_formatted_string("123456".to_string()), "123,456");
        assert_eq!(display_formatted_string("1234567".to_string()), "1,234,567");
        assert_eq!(
            display_formatted_string("1234567890".to_string()),
            "1,234,567,890"
        );
    }

    #[test]
    fn test_display_formatted_string_decimals() {
        assert_eq!(display_formatted_string("1.5".to_string()), "1.5");
        assert_eq!(display_formatted_string("1.50".to_string()), "1.50");
        assert_eq!(display_formatted_string("1.500".to_string()), "1.500");
        assert_eq!(display_formatted_string("1.23".to_string()), "1.23");
        assert_eq!(display_formatted_string("1.230".to_string()), "1.230");
    }

    #[test]
    fn test_display_formatted_string_trailing_zeros_preserved() {
        assert_eq!(display_formatted_string("1.00".to_string()), "1.00");
        assert_eq!(display_formatted_string("1.000".to_string()), "1.000");
        assert_eq!(display_formatted_string("100.00".to_string()), "100.00");
        assert_eq!(display_formatted_string("1000.00".to_string()), "1,000.00");
    }

    #[test]
    fn test_display_formatted_string_negative() {
        assert_eq!(display_formatted_string("-1".to_string()), "-1");
        assert_eq!(display_formatted_string("-1000".to_string()), "-1,000");
        assert_eq!(
            display_formatted_string("-1234.56".to_string()),
            "-1,234.56"
        );
    }

    #[test]
    fn test_display_f64_tiered_decimals() {
        assert_eq!(display(&12345.67), "12,346");
        assert_eq!(display(&1234.56), "1,234.6");
        assert_eq!(display(&123.456), "123.46");
        assert_eq!(display(&15.4567), "15.457");
        assert_eq!(display(&1.23456), "1.2346");
        assert_eq!(display(&0.123456), "0.1235");
        assert_eq!(display(&0.012345), "0.01235");
    }

    #[test]
    fn test_display_f64_tier_boundaries() {
        assert_eq!(display(&10000.0), "10,000");
        assert_eq!(display(&9999.99), "10,000.0");
        assert_eq!(display(&1000.0), "1,000.0");
        assert_eq!(display(&999.999), "1,000.00");
        assert_eq!(display(&100.0), "100.00");
        assert_eq!(display(&99.9999), "100.000");
        assert_eq!(display(&10.0), "10.000");
        assert_eq!(display(&9.87654), "9.8765");
        assert_eq!(display(&1.0), "1.0000");
        assert_eq!(display(&0.999999), "1.0000");
        assert_eq!(display(&0.1), "0.1000");
        assert_eq!(display(&0.099999), "0.10000");
    }

    #[test]
    fn test_display_f64_small_values_cap_at_five_decimals() {
        assert_eq!(display(&0.01), "0.01000");
        assert_eq!(display(&0.00003), "0.00003");
        assert_eq!(display(&0.000004), "0.00000");
        assert_eq!(display(&0.000000001), "0.00000");
    }

    #[test]
    fn test_display_f64_zero() {
        assert_eq!(display(&0.0), "0.00000");
    }

    #[test]
    fn test_display_f64_negative_values() {
        assert_eq!(display(&-1234.56), "-1,234.6");
        assert_eq!(display(&-15.4567), "-15.457");
        assert_eq!(display(&-0.00003), "-0.00003");
    }

    #[test]
    fn test_display_with_decimals_explicit() {
        assert_eq!(display_with_decimals(&1.0, 0), "1");
        assert_eq!(display_with_decimals(&1.0, 2), "1.00");
        assert_eq!(display_with_decimals(&1.5, 2), "1.50");
        assert_eq!(display_with_decimals(&1.234, 2), "1.23");
        assert_eq!(display_with_decimals(&1.235, 2), "1.24");
    }

    #[test]
    fn test_display_with_decimals_large_numbers() {
        assert_eq!(display_with_decimals(&1234567.89, 2), "1,234,567.89");
        assert_eq!(display_with_decimals(&1234567.0, 0), "1,234,567");
    }

    #[test]
    fn test_to_decimal_value() {
        assert_eq!(to_decimal_value(1_000_000_000, 9), 1.0);
        assert_eq!(to_decimal_value(1_500_000_000, 9), 1.5);
        assert_eq!(to_decimal_value(1_000_000, 6), 1.0);
        assert_eq!(to_decimal_value(500_000, 6), 0.5);
        assert_eq!(to_decimal_value(0, 9), 0.0);
    }

    #[test]
    fn test_from_decimal_value() {
        assert_eq!(from_decimal_value(1.0, 9), 1_000_000_000);
        assert_eq!(from_decimal_value(1.5, 9), 1_500_000_000);
        assert_eq!(from_decimal_value(1.0, 6), 1_000_000);
        assert_eq!(from_decimal_value(0.5, 6), 500_000);
        assert_eq!(from_decimal_value(0.0, 9), 0);
    }

    #[test]
    fn test_decimal_value_roundtrip() {
        let original: u64 = 123_456_789;
        let decimals: u64 = 9;
        let as_float = to_decimal_value(original, decimals);
        let back = from_decimal_value(as_float, decimals);
        assert_eq!(back, original);
    }
}
