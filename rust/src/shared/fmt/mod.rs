//! Formatting utilities for human-readable number display.

const DEFAULT_DECIMALS: usize = 2;
const TINY_SIGNIFICANT_DIGITS: usize = 3;
const MAX_STANDARD_DECIMALS: usize = 8;
const SUBSCRIPT_SIGNIFICANT_DIGITS: usize = 4;

enum DisplayFormat {
    Standard {
        decimals: usize,
        trim_trailing_zeros: bool,
    },
    Subscript,
}

fn display_format(
    is_zero: bool,
    rounds_to_default_nonzero: bool,
    leading_zeros: usize,
) -> DisplayFormat {
    if is_zero || rounds_to_default_nonzero {
        return DisplayFormat::Standard {
            decimals: DEFAULT_DECIMALS,
            trim_trailing_zeros: false,
        };
    }

    if leading_zeros + 1 > MAX_STANDARD_DECIMALS {
        return DisplayFormat::Subscript;
    }

    DisplayFormat::Standard {
        decimals: (leading_zeros + TINY_SIGNIFICANT_DIGITS).min(MAX_STANDARD_DECIMALS),
        trim_trailing_zeros: true,
    }
}

fn trim_trailing_fraction_zeros(formatted: String) -> String {
    if !formatted.contains('.') {
        return formatted;
    }

    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

pub mod decimal;
pub mod num;
