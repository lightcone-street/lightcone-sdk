//! Decimal formatting utilities for human-readable display.
//!
//! Handles `rust_decimal::Decimal` values with automatic decimal-place detection,
//! abbreviated suffixes (K/M/B/T), and conversion to on-chain base units.

use rust_decimal::prelude::*;
use std::sync::OnceLock;

/// The compact visual and fully expanded spellings of one unchanged Decimal.
///
/// Both strings are derived from the same normalized value. `compact` may count
/// a long run of leading fractional zeros with Unicode subscripts, while
/// `expanded` always contains every digit and is suitable for accessible text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactDecimalDisplay {
    /// The exact visual spelling, with long leading fractional zero runs compacted.
    pub compact: String,
    /// The exact grouped spelling with every fractional digit written out.
    pub expanded: String,
}

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

/// Formats a Decimal without changing its numeric value.
///
/// Integer digits are grouped, insignificant trailing fractional zeros are
/// removed by Decimal normalization, and zero is rendered as `0`. A run of six
/// or more zeros immediately after the decimal point of a value between -1 and
/// 1 is represented in `compact` as `0` followed by a Unicode subscript count;
/// `expanded` retains the full run. For example, `0.000000123456` becomes compact
/// `0.0₆123456` and expanded `0.000000123456`.
pub fn display_exact(value: &Decimal) -> ExactDecimalDisplay {
    let expanded = super::num::display_formatted_string(value.normalize().to_string());
    let compact = compact_leading_fractional_zeros(&expanded);
    ExactDecimalDisplay { compact, expanded }
}

/// Compacts only a tiny value's six-or-more leading fractional zero run.
fn compact_leading_fractional_zeros(expanded: &str) -> String {
    let Some((integer, fraction)) = expanded.split_once('.') else {
        return expanded.to_string();
    };
    if integer != "0" && integer != "-0" {
        return expanded.to_string();
    }
    let zero_count = fraction.bytes().take_while(|byte| *byte == b'0').count();
    if zero_count < 6 || zero_count == fraction.len() {
        return expanded.to_string();
    }

    format!(
        "{integer}.0{}{}",
        subscript_count(zero_count),
        &fraction[zero_count..]
    )
}

/// Writes a decimal zero-count with Unicode subscript digits.
fn subscript_count(count: usize) -> String {
    count
        .to_string()
        .chars()
        .map(|digit| match digit {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            other => other,
        })
        .collect()
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

/// Format a `Decimal` as a percentage with exactly 2 decimal places (truncated).
///
/// When `padding` is true (default), always shows 2 decimal places (e.g. "12.30").
/// When false, trailing zeros are trimmed (e.g. "12.3").
pub fn display_pct(value: &Decimal, padding: Option<bool>) -> String {
    let padding = padding.unwrap_or(true);
    let truncated = value.round_dp_with_strategy(2, RoundingStrategy::ToZero);

    if padding {
        super::num::display_formatted_string(format!("{:.2}", truncated))
    } else {
        let normalized = truncated.normalize();
        super::num::display_formatted_string(format!("{}", normalized))
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
    /// Proves exact display groups and normalizes without changing significant digits.
    fn test_display_exact_preserves_value() {
        assert_eq!(
            display_exact(&dec("1234.500000")),
            ExactDecimalDisplay {
                compact: "1,234.5".to_string(),
                expanded: "1,234.5".to_string(),
            }
        );
        assert_eq!(
            display_exact(&dec("1.0000009")),
            ExactDecimalDisplay {
                compact: "1.0000009".to_string(),
                expanded: "1.0000009".to_string(),
            }
        );
        assert_eq!(
            display_exact(&Decimal::ZERO),
            ExactDecimalDisplay {
                compact: "0".to_string(),
                expanded: "0".to_string(),
            }
        );
    }

    #[test]
    /// Locks the six-zero compact grammar while retaining the expanded source of truth.
    fn test_display_exact_compacts_leading_fractional_zeros() {
        assert_eq!(
            display_exact(&dec("0.00000123456")),
            ExactDecimalDisplay {
                compact: "0.00000123456".to_string(),
                expanded: "0.00000123456".to_string(),
            }
        );
        assert_eq!(
            display_exact(&dec("0.000000123456")),
            ExactDecimalDisplay {
                compact: "0.0₆123456".to_string(),
                expanded: "0.000000123456".to_string(),
            }
        );
        assert_eq!(
            display_exact(&dec("-0.000000123456")),
            ExactDecimalDisplay {
                compact: "-0.0₆123456".to_string(),
                expanded: "-0.000000123456".to_string(),
            }
        );
        assert_eq!(
            display_exact(&dec("0.0000000000123456")),
            ExactDecimalDisplay {
                compact: "0.0₁₀123456".to_string(),
                expanded: "0.0000000000123456".to_string(),
            }
        );
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

    #[test]
    fn test_display_pct_truncation() {
        assert_eq!(display_pct(&dec("12.345"), None), "12.34");
        assert_eq!(display_pct(&dec("12.999"), None), "12.99");
        assert_eq!(display_pct(&dec("99.999"), None), "99.99");
    }

    #[test]
    fn test_display_pct_padding_true() {
        assert_eq!(display_pct(&dec("12.3"), None), "12.30");
        assert_eq!(display_pct(&dec("12"), None), "12.00");
        assert_eq!(display_pct(&Decimal::ZERO, None), "0.00");
    }

    #[test]
    fn test_display_pct_padding_false() {
        assert_eq!(display_pct(&dec("12.345"), Some(false)), "12.34");
        assert_eq!(display_pct(&dec("12.3"), Some(false)), "12.3");
        assert_eq!(display_pct(&dec("12"), Some(false)), "12");
        assert_eq!(display_pct(&Decimal::ZERO, Some(false)), "0");
    }

    #[test]
    fn test_display_pct_negative() {
        assert_eq!(display_pct(&dec("-3.456"), None), "-3.45");
        assert_eq!(display_pct(&dec("-3.4"), None), "-3.40");
        assert_eq!(display_pct(&dec("-3.456"), Some(false)), "-3.45");
    }
}
