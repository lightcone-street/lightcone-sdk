//! Formatting utilities for human-readable number display.

const DEFAULT_DECIMALS: usize = 2;
const TINY_SIGNIFICANT_DIGITS: usize = 3;
const MAX_STANDARD_DECIMALS: usize = 8;
const SUBSCRIPT_SIGNIFICANT_DIGITS: usize = 4;

pub mod decimal;
pub mod num;
