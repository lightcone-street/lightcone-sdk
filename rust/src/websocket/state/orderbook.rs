//! Local orderbook state management.
//!
//! Maintains a local copy of the orderbook state, applying deltas from
//! WebSocket updates.
//!
//! Note: Internally uses String keys/values to match the String-based API.
//! For numeric comparisons, parse the strings as needed.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::str::FromStr;

use rust_decimal::Decimal;

use crate::websocket::error::WebSocketError;
use crate::websocket::types::{BookUpdateData, PriceLevel};

/// A price key wrapper that provides numeric ordering for string prices.
///
/// This ensures prices are sorted numerically (e.g., "0.5" > "0.10") rather than
/// lexicographically (where "0.10" > "0.5" as strings).
#[derive(Debug, Clone, Eq, PartialEq)]
struct PriceKey(String);

impl PriceKey {
    fn new(price: String) -> Self {
        Self(price)
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl Ord for PriceKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_dec = Decimal::from_str(&self.0).unwrap_or(Decimal::ZERO);
        let other_dec = Decimal::from_str(&other.0).unwrap_or(Decimal::ZERO);
        self_dec.cmp(&other_dec)
    }
}

impl PartialOrd for PriceKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Check if a size string represents zero using precise decimal comparison.
fn is_zero_size(s: &str) -> bool {
    Decimal::from_str(s)
        .map(|v| v.is_zero())
        .unwrap_or(false)
}

/// Local orderbook state
#[derive(Debug, Clone)]
pub struct LocalOrderbook {
    /// Orderbook identifier
    pub orderbook_id: String,
    /// Bid levels (price -> size), sorted numerically by price
    bids: BTreeMap<PriceKey, String>,
    /// Ask levels (price -> size), sorted numerically by price
    asks: BTreeMap<PriceKey, String>,
    /// Expected next sequence number
    expected_seq: u64,
    /// Whether initial snapshot has been received
    has_snapshot: bool,
    /// Last update timestamp
    last_timestamp: Option<String>,
}

impl LocalOrderbook {
    /// Create a new empty orderbook
    pub fn new(orderbook_id: String) -> Self {
        Self {
            orderbook_id,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            expected_seq: 0,
            has_snapshot: false,
            last_timestamp: None,
        }
    }

    /// Apply a snapshot (full orderbook state)
    pub fn apply_snapshot(&mut self, update: &BookUpdateData) {
        // Clear existing state
        self.bids.clear();
        self.asks.clear();

        // Apply all levels
        for level in &update.bids {
            if !is_zero_size(&level.size) {
                self.bids
                    .insert(PriceKey::new(level.price.clone()), level.size.clone());
            }
        }

        for level in &update.asks {
            if !is_zero_size(&level.size) {
                self.asks
                    .insert(PriceKey::new(level.price.clone()), level.size.clone());
            }
        }

        self.expected_seq = update.seq + 1;
        self.has_snapshot = true;
        self.last_timestamp = Some(update.timestamp.clone());
    }

    /// Apply a delta update
    ///
    /// Returns an error if a sequence gap is detected.
    pub fn apply_delta(&mut self, update: &BookUpdateData) -> Result<(), WebSocketError> {
        // Check sequence number
        if update.seq != self.expected_seq {
            return Err(WebSocketError::SequenceGap {
                expected: self.expected_seq,
                received: update.seq,
            });
        }

        // Apply bid updates
        for level in &update.bids {
            let key = PriceKey::new(level.price.clone());
            if is_zero_size(&level.size) {
                self.bids.remove(&key);
            } else {
                self.bids.insert(key, level.size.clone());
            }
        }

        // Apply ask updates
        for level in &update.asks {
            let key = PriceKey::new(level.price.clone());
            if is_zero_size(&level.size) {
                self.asks.remove(&key);
            } else {
                self.asks.insert(key, level.size.clone());
            }
        }

        self.expected_seq = update.seq + 1;
        self.last_timestamp = Some(update.timestamp.clone());
        Ok(())
    }

    /// Apply an update (snapshot or delta)
    pub fn apply_update(&mut self, update: &BookUpdateData) -> Result<(), WebSocketError> {
        if update.is_snapshot {
            self.apply_snapshot(update);
            Ok(())
        } else {
            self.apply_delta(update)
        }
    }

    /// Get all bid levels sorted by price (descending - highest first)
    pub fn get_bids(&self) -> Vec<PriceLevel> {
        self.bids
            .iter()
            .rev()
            .map(|(price, size)| PriceLevel {
                side: "bid".to_string(),
                price: price.as_str().to_string(),
                size: size.clone(),
            })
            .collect()
    }

    /// Get all ask levels sorted by price (ascending - lowest first)
    pub fn get_asks(&self) -> Vec<PriceLevel> {
        self.asks
            .iter()
            .map(|(price, size)| PriceLevel {
                side: "ask".to_string(),
                price: price.as_str().to_string(),
                size: size.clone(),
            })
            .collect()
    }

    /// Get top N bid levels (highest prices first)
    pub fn get_top_bids(&self, n: usize) -> Vec<PriceLevel> {
        self.bids
            .iter()
            .rev()
            .take(n)
            .map(|(price, size)| PriceLevel {
                side: "bid".to_string(),
                price: price.as_str().to_string(),
                size: size.clone(),
            })
            .collect()
    }

    /// Get top N ask levels (lowest prices first)
    pub fn get_top_asks(&self, n: usize) -> Vec<PriceLevel> {
        self.asks
            .iter()
            .take(n)
            .map(|(price, size)| PriceLevel {
                side: "ask".to_string(),
                price: price.as_str().to_string(),
                size: size.clone(),
            })
            .collect()
    }

    /// Get the best bid (highest bid price) as (price_string, size_string)
    pub fn best_bid(&self) -> Option<(String, String)> {
        self.bids
            .iter()
            .next_back()
            .map(|(p, s)| (p.as_str().to_string(), s.clone()))
    }

    /// Get the best ask (lowest ask price) as (price_string, size_string)
    pub fn best_ask(&self) -> Option<(String, String)> {
        self.asks
            .iter()
            .next()
            .map(|(p, s)| (p.as_str().to_string(), s.clone()))
    }

    /// Get the spread as a string (best_ask - best_bid)
    /// Uses Decimal for precise calculation to preserve backend precision.
    pub fn spread(&self) -> Option<String> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => {
                let bid_dec = Decimal::from_str(&bid).ok()?;
                let ask_dec = Decimal::from_str(&ask).ok()?;
                if ask_dec > bid_dec {
                    Some((ask_dec - bid_dec).to_string())
                } else {
                    Some(Decimal::ZERO.to_string())
                }
            }
            _ => None,
        }
    }

    /// Get the midpoint price as a string
    /// Uses Decimal for precise calculation to preserve backend precision.
    pub fn midpoint(&self) -> Option<String> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => {
                let bid_dec = Decimal::from_str(&bid).ok()?;
                let ask_dec = Decimal::from_str(&ask).ok()?;
                let two = Decimal::from(2);
                Some(((bid_dec + ask_dec) / two).to_string())
            }
            _ => None,
        }
    }

    /// Get size at a specific bid price
    pub fn bid_size_at(&self, price: &str) -> Option<String> {
        self.bids.get(&PriceKey::new(price.to_string())).cloned()
    }

    /// Get size at a specific ask price
    pub fn ask_size_at(&self, price: &str) -> Option<String> {
        self.asks.get(&PriceKey::new(price.to_string())).cloned()
    }

    /// Get total bid depth (sum of all bid sizes)
    /// Uses Decimal for precise calculation to preserve backend precision.
    pub fn total_bid_depth(&self) -> Decimal {
        self.bids
            .values()
            .filter_map(|s| Decimal::from_str(s).ok())
            .fold(Decimal::ZERO, |acc, x| acc + x)
    }

    /// Get total ask depth (sum of all ask sizes)
    /// Uses Decimal for precise calculation to preserve backend precision.
    pub fn total_ask_depth(&self) -> Decimal {
        self.asks
            .values()
            .filter_map(|s| Decimal::from_str(s).ok())
            .fold(Decimal::ZERO, |acc, x| acc + x)
    }

    /// Number of bid levels
    pub fn bid_count(&self) -> usize {
        self.bids.len()
    }

    /// Number of ask levels
    pub fn ask_count(&self) -> usize {
        self.asks.len()
    }

    /// Whether the orderbook has received its initial snapshot
    pub fn has_snapshot(&self) -> bool {
        self.has_snapshot
    }

    /// Current expected sequence number
    pub fn expected_sequence(&self) -> u64 {
        self.expected_seq
    }

    /// Last update timestamp
    pub fn last_timestamp(&self) -> Option<&str> {
        self.last_timestamp.as_deref()
    }

    /// Clear the orderbook state (for resync)
    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.expected_seq = 0;
        self.has_snapshot = false;
        self.last_timestamp = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_snapshot() -> BookUpdateData {
        BookUpdateData {
            orderbook_id: "test".to_string(),
            timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            seq: 0,
            bids: vec![
                PriceLevel {
                    side: "bid".to_string(),
                    price: "0.500000".to_string(),
                    size: "0.001000".to_string(),
                },
                PriceLevel {
                    side: "bid".to_string(),
                    price: "0.490000".to_string(),
                    size: "0.002000".to_string(),
                },
            ],
            asks: vec![
                PriceLevel {
                    side: "ask".to_string(),
                    price: "0.510000".to_string(),
                    size: "0.000500".to_string(),
                },
                PriceLevel {
                    side: "ask".to_string(),
                    price: "0.520000".to_string(),
                    size: "0.001500".to_string(),
                },
            ],
            is_snapshot: true,
            resync: false,
            message: None,
        }
    }

    #[test]
    fn test_apply_snapshot() {
        let mut book = LocalOrderbook::new("test".to_string());
        let snapshot = create_snapshot();

        book.apply_snapshot(&snapshot);

        assert!(book.has_snapshot());
        assert_eq!(book.expected_sequence(), 1);
        assert_eq!(book.bid_count(), 2);
        assert_eq!(book.ask_count(), 2);
        assert_eq!(book.best_bid(), Some(("0.500000".to_string(), "0.001000".to_string())));
        assert_eq!(book.best_ask(), Some(("0.510000".to_string(), "0.000500".to_string())));
    }

    #[test]
    fn test_apply_delta() {
        let mut book = LocalOrderbook::new("test".to_string());
        book.apply_snapshot(&create_snapshot());

        let delta = BookUpdateData {
            orderbook_id: "test".to_string(),
            timestamp: "2024-01-01T00:00:00.050Z".to_string(),
            seq: 1,
            bids: vec![PriceLevel {
                side: "bid".to_string(),
                price: "0.500000".to_string(),
                size: "0.001500".to_string(), // Updated size
            }],
            asks: vec![PriceLevel {
                side: "ask".to_string(),
                price: "0.510000".to_string(),
                size: "0".to_string(), // Remove level
            }],
            is_snapshot: false,
            resync: false,
            message: None,
        };

        book.apply_delta(&delta).unwrap();

        assert_eq!(book.best_bid(), Some(("0.500000".to_string(), "0.001500".to_string())));
        assert_eq!(book.best_ask(), Some(("0.520000".to_string(), "0.001500".to_string())));
        assert_eq!(book.ask_count(), 1);
    }

    #[test]
    fn test_sequence_gap_detection() {
        let mut book = LocalOrderbook::new("test".to_string());
        book.apply_snapshot(&create_snapshot());

        let delta = BookUpdateData {
            orderbook_id: "test".to_string(),
            timestamp: "2024-01-01T00:00:00.050Z".to_string(),
            seq: 5, // Gap!
            bids: vec![],
            asks: vec![],
            is_snapshot: false,
            resync: false,
            message: None,
        };

        let result = book.apply_delta(&delta);
        assert!(matches!(result, Err(WebSocketError::SequenceGap { .. })));
    }

    #[test]
    fn test_spread_and_midpoint() {
        let mut book = LocalOrderbook::new("test".to_string());
        book.apply_snapshot(&create_snapshot());

        // Decimal preserves precision from input strings
        assert_eq!(book.spread(), Some("0.010000".to_string()));
        assert_eq!(book.midpoint(), Some("0.505000".to_string()));
    }

    #[test]
    fn test_depth() {
        let mut book = LocalOrderbook::new("test".to_string());
        book.apply_snapshot(&create_snapshot());

        // Returns Decimal now for precise calculations
        assert_eq!(book.total_bid_depth(), Decimal::from_str("0.003").unwrap());
        assert_eq!(book.total_ask_depth(), Decimal::from_str("0.002").unwrap());
    }

    #[test]
    fn test_variable_precision_price_sorting() {
        // This test ensures prices are sorted numerically, not lexicographically.
        // Lexicographic: "0.10" > "0.5" (because '1' > '0' in second char)
        // Numeric: "0.5" > "0.10" (because 0.5 > 0.1)
        let mut book = LocalOrderbook::new("test".to_string());

        let snapshot = BookUpdateData {
            orderbook_id: "test".to_string(),
            timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            seq: 0,
            bids: vec![
                PriceLevel {
                    side: "bid".to_string(),
                    price: "0.10".to_string(), // 0.1
                    size: "1.0".to_string(),
                },
                PriceLevel {
                    side: "bid".to_string(),
                    price: "0.5".to_string(), // 0.5 - should be best bid
                    size: "2.0".to_string(),
                },
                PriceLevel {
                    side: "bid".to_string(),
                    price: "0.100000".to_string(), // Also 0.1, same as first
                    size: "3.0".to_string(),
                },
            ],
            asks: vec![
                PriceLevel {
                    side: "ask".to_string(),
                    price: "0.9".to_string(), // 0.9
                    size: "1.0".to_string(),
                },
                PriceLevel {
                    side: "ask".to_string(),
                    price: "0.51".to_string(), // 0.51 - should be best ask
                    size: "2.0".to_string(),
                },
            ],
            is_snapshot: true,
            resync: false,
            message: None,
        };

        book.apply_snapshot(&snapshot);

        // Best bid should be 0.5 (highest), not "0.10" (which would be lexicographically highest)
        let best_bid = book.best_bid().unwrap();
        assert_eq!(best_bid.0, "0.5", "Best bid should be 0.5, not {}", best_bid.0);

        // Best ask should be 0.51 (lowest), not "0.9" (which would be lexicographically lowest)
        let best_ask = book.best_ask().unwrap();
        assert_eq!(
            best_ask.0, "0.51",
            "Best ask should be 0.51, not {}",
            best_ask.0
        );

        // Verify bid ordering: should be [0.5, 0.100000] (descending by numeric value)
        // Note: "0.10" and "0.100000" both represent 0.1, so one overwrites the other
        let bids = book.get_bids();
        assert_eq!(bids.len(), 2); // 0.5 and 0.1 (one of the 0.1s)
        assert_eq!(bids[0].price, "0.5"); // Highest first
    }

    #[test]
    fn test_is_zero_size() {
        // Test the is_zero_size helper
        assert!(super::is_zero_size("0"));
        assert!(super::is_zero_size("0.0"));
        assert!(super::is_zero_size("0.000000"));
        assert!(super::is_zero_size("0.00000000000"));
        assert!(!super::is_zero_size("0.001"));
        assert!(!super::is_zero_size("1"));
        assert!(!super::is_zero_size("0.0000001"));
    }
}
