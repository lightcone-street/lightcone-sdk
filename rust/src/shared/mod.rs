//! Shared utilities and types used across API and WebSocket modules.

pub mod price;
pub mod scaling;
pub mod types;

// Re-export commonly used items
pub use price::{format_decimal, parse_decimal};
pub use scaling::{scale_price_size, OrderbookDecimals, ScaledAmounts, ScalingError};
pub use types::*;

/// Derive orderbook ID from base and quote token pubkeys.
///
/// Format: `{base_token[0:8]}_{quote_token[0:8]}`
///
/// # Example
///
/// ```rust
/// use lightcone_sdk::shared::derive_orderbook_id;
///
/// let orderbook_id = derive_orderbook_id(
///     "7BgBvyjrZX1YKz4oh9mjb8ZScatkkwb8DzFx7LoiVkM3",
///     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
/// );
/// assert_eq!(orderbook_id, "7BgBvyjr_EPjFWdd5");
/// ```
pub fn derive_orderbook_id(base_token: &str, quote_token: &str) -> String {
    let base_prefix = &base_token[..8.min(base_token.len())];
    let quote_prefix = &quote_token[..8.min(quote_token.len())];
    format!("{}_{}", base_prefix, quote_prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_orderbook_id() {
        let orderbook_id = derive_orderbook_id(
            "7BgBvyjrZX1YKz4oh9mjb8ZScatkkwb8DzFx7LoiVkM3",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        );
        assert_eq!(orderbook_id, "7BgBvyjr_EPjFWdd5");
    }

    #[test]
    fn test_derive_orderbook_id_short_tokens() {
        // Test with shorter token strings
        let orderbook_id = derive_orderbook_id("ABCD", "XYZ");
        assert_eq!(orderbook_id, "ABCD_XYZ");
    }

    #[test]
    fn test_derive_orderbook_id_exact_length() {
        let orderbook_id = derive_orderbook_id("12345678", "ABCDEFGH");
        assert_eq!(orderbook_id, "12345678_ABCDEFGH");
    }
}
