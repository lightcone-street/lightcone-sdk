//! Shared display precision tiers.

pub(super) const SMALL_VALUE_DECIMALS: usize = 5;

pub(super) const DISPLAY_DECIMAL_TIERS: [(&str, usize); 5] =
    [("10000", 0), ("1000", 1), ("100", 2), ("10", 3), ("0.1", 4)];

pub(super) fn display_decimals_by(matches_threshold: impl Fn(&str) -> bool) -> usize {
    DISPLAY_DECIMAL_TIERS
        .iter()
        .find(|(threshold, _)| matches_threshold(threshold))
        .map(|(_, decimals)| *decimals)
        .unwrap_or(SMALL_VALUE_DECIMALS)
}
