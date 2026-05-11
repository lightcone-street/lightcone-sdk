//! Formatting utilities for human-readable number display.

use rust_decimal::Decimal;

fn display_decimals_f64(abs_value: f64) -> usize {
    if abs_value >= 10_000.0 {
        0
    } else if abs_value >= 1_000.0 {
        1
    } else if abs_value >= 100.0 {
        2
    } else if abs_value >= 10.0 {
        3
    } else if abs_value >= 0.1 {
        4
    } else {
        5
    }
}

fn display_decimals_decimal(abs_value: &Decimal) -> usize {
    if abs_value >= &Decimal::from(10_000) {
        0
    } else if abs_value >= &Decimal::from(1_000) {
        1
    } else if abs_value >= &Decimal::from(100) {
        2
    } else if abs_value >= &Decimal::from(10) {
        3
    } else if abs_value >= &Decimal::new(1, 1) {
        4
    } else {
        5
    }
}

pub mod decimal;
pub mod num;
