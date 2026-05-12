//! Shared display precision tiers.

use rust_decimal::Decimal;

pub(super) const SMALL_VALUE_DECIMALS: usize = 5;

pub(super) const DISPLAY_DECIMAL_TIERS: [DisplayDecimalTier; 5] = [
    DisplayDecimalTier::new(10_000, 0, 0),
    DisplayDecimalTier::new(1_000, 0, 1),
    DisplayDecimalTier::new(100, 0, 2),
    DisplayDecimalTier::new(10, 0, 3),
    DisplayDecimalTier::new(1, 1, 4),
];

#[derive(Clone, Copy)]
pub(super) struct DisplayDecimalTier {
    mantissa: i64,
    scale: u32,
    pub decimals: usize,
}

impl DisplayDecimalTier {
    const fn new(mantissa: i64, scale: u32, decimals: usize) -> Self {
        Self {
            mantissa,
            scale,
            decimals,
        }
    }

    pub fn threshold_f64(self) -> f64 {
        self.mantissa as f64 / 10f64.powi(self.scale as i32)
    }

    pub fn threshold_decimal(self) -> Decimal {
        Decimal::new(self.mantissa, self.scale)
    }
}

pub(super) fn display_decimals_by(matches_tier: impl Fn(DisplayDecimalTier) -> bool) -> usize {
    DISPLAY_DECIMAL_TIERS
        .iter()
        .copied()
        .find(|tier| matches_tier(*tier))
        .map(|tier| tier.decimals)
        .unwrap_or(SMALL_VALUE_DECIMALS)
}
