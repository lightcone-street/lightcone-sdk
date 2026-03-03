//! Number formatting utilities for human-readable display.
//!
//! Handles f64 values with automatic decimal-place detection and comma separators.
//! For `Decimal` formatting, use the `decimal` sibling module.

/// Trims trailing zeros, adds thousands separators.
pub fn display_formatted_string(formatted: String) -> String {
    let trimmed = if formatted.contains('.') {
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        formatted
    };

    let parts = trimmed.split(".").collect::<Vec<_>>();

    let integer_part = parts[0]
        .chars()
        .rev()
        .collect::<String>()
        .as_bytes()
        .chunks(3)
        .map(|c| std::str::from_utf8(c).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(",")
        .chars()
        .rev()
        .collect::<String>();

    let integer_part = integer_part
        .strip_prefix("-,")
        .or_else(|| integer_part.strip_prefix(","))
        .unwrap_or(&integer_part)
        .to_string();

    if parts.len() > 1 {
        format!("{}.{}", integer_part, parts[1])
    } else {
        integer_part
    }
}

fn get_decimal_places(value: f64) -> usize {
    let abs_value = value.abs();

    if abs_value >= 100.0 {
        return 0;
    }

    if abs_value >= 1.0 {
        return 2;
    }

    if abs_value == 0.0 {
        return 2;
    }

    let exponent = abs_value.log10().floor().abs() as usize;
    (exponent + 2).min(8)
}

/// Format an f64 for display with auto-detected decimal places.
pub fn display(amount: &f64) -> String {
    display_with_decimals(amount, get_decimal_places(*amount))
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
        assert_eq!(display_formatted_string("1.50".to_string()), "1.5");
        assert_eq!(display_formatted_string("1.500".to_string()), "1.5");
        assert_eq!(display_formatted_string("1.23".to_string()), "1.23");
        assert_eq!(display_formatted_string("1.230".to_string()), "1.23");
    }

    #[test]
    fn test_display_formatted_string_trailing_zeros_trimmed() {
        assert_eq!(display_formatted_string("1.00".to_string()), "1");
        assert_eq!(display_formatted_string("1.000".to_string()), "1");
        assert_eq!(display_formatted_string("100.00".to_string()), "100");
        assert_eq!(display_formatted_string("1000.00".to_string()), "1,000");
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
    fn test_display_f64_large() {
        assert_eq!(display(&100.0), "100");
        assert_eq!(display(&1234.56), "1,235");
        assert_eq!(display(&999999.0), "999,999");
    }

    #[test]
    fn test_display_f64_medium() {
        assert_eq!(display(&1.0), "1");
        assert_eq!(display(&1.5), "1.5");
        assert_eq!(display(&1.23), "1.23");
        assert_eq!(display(&15.456), "15.46");
        assert_eq!(display(&99.999), "100");
    }

    #[test]
    fn test_display_f64_small() {
        assert_eq!(display(&0.1), "0.1");
        assert_eq!(display(&0.123), "0.123");
        assert_eq!(display(&0.01), "0.01");
        assert_eq!(display(&0.0123), "0.0123");
    }

    #[test]
    fn test_display_f64_zero() {
        assert_eq!(display(&0.0), "0");
    }

    #[test]
    fn test_display_with_decimals_explicit() {
        assert_eq!(display_with_decimals(&1.0, 0), "1");
        assert_eq!(display_with_decimals(&1.0, 2), "1");
        assert_eq!(display_with_decimals(&1.5, 2), "1.5");
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
