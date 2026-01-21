//! Local orderbook state management.
//!
//! Maintains a local copy of the orderbook state, applying deltas from
//! WebSocket updates.
//!
//! Note: Internally uses String keys/values to match the String-based API.
//! For numeric comparisons, parse the strings as needed.

use std::collections::BTreeMap;

use crate::websocket::error::WebSocketError;
use crate::websocket::types::{BookUpdateData, PriceLevel};

/// Local orderbook state
#[derive(Debug, Clone)]
pub struct LocalOrderbook {
    /// Orderbook identifier
    pub orderbook_id: String,
    /// Bid levels (price string -> size string), sorted descending by price
    bids: BTreeMap<String, String>,
    /// Ask levels (price string -> size string), sorted ascending by price
    asks: BTreeMap<String, String>,
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
            if level.size.parse::<f64>().map(|v| v != 0.0).unwrap_or(false) {
                self.bids.insert(level.price.clone(), level.size.clone());
            }
        }

        for level in &update.asks {
            if level.size.parse::<f64>().map(|v| v != 0.0).unwrap_or(false) {
                self.asks.insert(level.price.clone(), level.size.clone());
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
            if level.size.parse::<f64>().map(|v| v == 0.0).unwrap_or(false) {
                self.bids.remove(&level.price);
            } else {
                self.bids.insert(level.price.clone(), level.size.clone());
            }
        }

        // Apply ask updates
        for level in &update.asks {
            if level.size.parse::<f64>().map(|v| v == 0.0).unwrap_or(false) {
                self.asks.remove(&level.price);
            } else {
                self.asks.insert(level.price.clone(), level.size.clone());
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

    /// Get all bid levels sorted by price (descending by string comparison)
    pub fn get_bids(&self) -> Vec<PriceLevel> {
        self.bids
            .iter()
            .rev()
            .map(|(price, size)| PriceLevel {
                side: "bid".to_string(),
                price: price.clone(),
                size: size.clone(),
            })
            .collect()
    }

    /// Get all ask levels sorted by price (ascending by string comparison)
    pub fn get_asks(&self) -> Vec<PriceLevel> {
        self.asks
            .iter()
            .map(|(price, size)| PriceLevel {
                side: "ask".to_string(),
                price: price.clone(),
                size: size.clone(),
            })
            .collect()
    }

    /// Get top N bid levels
    pub fn get_top_bids(&self, n: usize) -> Vec<PriceLevel> {
        self.bids
            .iter()
            .rev()
            .take(n)
            .map(|(price, size)| PriceLevel {
                side: "bid".to_string(),
                price: price.clone(),
                size: size.clone(),
            })
            .collect()
    }

    /// Get top N ask levels
    pub fn get_top_asks(&self, n: usize) -> Vec<PriceLevel> {
        self.asks
            .iter()
            .take(n)
            .map(|(price, size)| PriceLevel {
                side: "ask".to_string(),
                price: price.clone(),
                size: size.clone(),
            })
            .collect()
    }

    /// Get the best bid (highest bid price) as (price_string, size_string)
    pub fn best_bid(&self) -> Option<(String, String)> {
        self.bids.iter().next_back().map(|(p, s)| (p.clone(), s.clone()))
    }

    /// Get the best ask (lowest ask price) as (price_string, size_string)
    pub fn best_ask(&self) -> Option<(String, String)> {
        self.asks.iter().next().map(|(p, s)| (p.clone(), s.clone()))
    }

    /// Get the spread as a string (best_ask - best_bid)
    /// Note: This parses as f64 for the calculation
    pub fn spread(&self) -> Option<String> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => {
                let bid_f: f64 = bid.parse().ok()?;
                let ask_f: f64 = ask.parse().ok()?;
                if ask_f > bid_f {
                    Some(format!("{:.6}", ask_f - bid_f))
                } else {
                    Some("0.000000".to_string())
                }
            }
            _ => None,
        }
    }

    /// Get the midpoint price as a string
    /// Note: This parses as f64 for the calculation
    pub fn midpoint(&self) -> Option<String> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => {
                let bid_f: f64 = bid.parse().ok()?;
                let ask_f: f64 = ask.parse().ok()?;
                Some(format!("{:.6}", (bid_f + ask_f) / 2.0))
            }
            _ => None,
        }
    }

    /// Get size at a specific bid price
    pub fn bid_size_at(&self, price: &str) -> Option<String> {
        self.bids.get(price).cloned()
    }

    /// Get size at a specific ask price
    pub fn ask_size_at(&self, price: &str) -> Option<String> {
        self.asks.get(price).cloned()
    }

    /// Get total bid depth (sum of all bid sizes)
    /// Note: This parses as f64 for the calculation
    pub fn total_bid_depth(&self) -> f64 {
        self.bids.values().filter_map(|s| s.parse::<f64>().ok()).sum()
    }

    /// Get total ask depth (sum of all ask sizes)
    /// Note: This parses as f64 for the calculation
    pub fn total_ask_depth(&self) -> f64 {
        self.asks.values().filter_map(|s| s.parse::<f64>().ok()).sum()
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

        assert_eq!(book.spread(), Some("0.010000".to_string()));
        assert_eq!(book.midpoint(), Some("0.505000".to_string()));
    }

    #[test]
    fn test_depth() {
        let mut book = LocalOrderbook::new("test".to_string());
        book.apply_snapshot(&create_snapshot());

        assert!((book.total_bid_depth() - 0.003).abs() < 0.0001);
        assert!((book.total_ask_depth() - 0.002).abs() < 0.0001);
    }
}
