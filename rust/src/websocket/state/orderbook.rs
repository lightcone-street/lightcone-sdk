//! Local orderbook state management.
//!
//! Maintains a local copy of the orderbook state, applying deltas from
//! WebSocket updates.

use std::collections::BTreeMap;

use crate::websocket::error::WebSocketError;
use crate::websocket::types::{BookUpdateData, PriceLevel};

/// Local orderbook state
#[derive(Debug, Clone)]
pub struct LocalOrderbook {
    /// Orderbook identifier
    pub orderbook_id: String,
    /// Bid levels (price -> size), sorted descending by price
    bids: BTreeMap<u64, u64>,
    /// Ask levels (price -> size), sorted ascending by price
    asks: BTreeMap<u64, u64>,
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
            if level.size > 0 {
                self.bids.insert(level.price, level.size);
            }
        }

        for level in &update.asks {
            if level.size > 0 {
                self.asks.insert(level.price, level.size);
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
            if level.size == 0 {
                self.bids.remove(&level.price);
            } else {
                self.bids.insert(level.price, level.size);
            }
        }

        // Apply ask updates
        for level in &update.asks {
            if level.size == 0 {
                self.asks.remove(&level.price);
            } else {
                self.asks.insert(level.price, level.size);
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

    /// Get all bid levels sorted by price (descending)
    pub fn get_bids(&self) -> Vec<PriceLevel> {
        self.bids
            .iter()
            .rev()
            .map(|(&price, &size)| PriceLevel {
                side: "bid".to_string(),
                price,
                size,
            })
            .collect()
    }

    /// Get all ask levels sorted by price (ascending)
    pub fn get_asks(&self) -> Vec<PriceLevel> {
        self.asks
            .iter()
            .map(|(&price, &size)| PriceLevel {
                side: "ask".to_string(),
                price,
                size,
            })
            .collect()
    }

    /// Get top N bid levels
    pub fn get_top_bids(&self, n: usize) -> Vec<PriceLevel> {
        self.bids
            .iter()
            .rev()
            .take(n)
            .map(|(&price, &size)| PriceLevel {
                side: "bid".to_string(),
                price,
                size,
            })
            .collect()
    }

    /// Get top N ask levels
    pub fn get_top_asks(&self, n: usize) -> Vec<PriceLevel> {
        self.asks
            .iter()
            .take(n)
            .map(|(&price, &size)| PriceLevel {
                side: "ask".to_string(),
                price,
                size,
            })
            .collect()
    }

    /// Get the best bid (highest bid price)
    pub fn best_bid(&self) -> Option<(u64, u64)> {
        self.bids.iter().next_back().map(|(&p, &s)| (p, s))
    }

    /// Get the best ask (lowest ask price)
    pub fn best_ask(&self) -> Option<(u64, u64)> {
        self.asks.iter().next().map(|(&p, &s)| (p, s))
    }

    /// Get the spread (best_ask - best_bid)
    pub fn spread(&self) -> Option<u64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => {
                if ask > bid {
                    Some(ask - bid)
                } else {
                    Some(0)
                }
            }
            _ => None,
        }
    }

    /// Get the midpoint price
    pub fn midpoint(&self) -> Option<u64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some((bid + ask) / 2),
            _ => None,
        }
    }

    /// Get size at a specific bid price
    pub fn bid_size_at(&self, price: u64) -> Option<u64> {
        self.bids.get(&price).copied()
    }

    /// Get size at a specific ask price
    pub fn ask_size_at(&self, price: u64) -> Option<u64> {
        self.asks.get(&price).copied()
    }

    /// Get total bid depth (sum of all bid sizes)
    pub fn total_bid_depth(&self) -> u64 {
        self.bids.values().sum()
    }

    /// Get total ask depth (sum of all ask sizes)
    pub fn total_ask_depth(&self) -> u64 {
        self.asks.values().sum()
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
                    price: 500000,
                    size: 1000,
                },
                PriceLevel {
                    side: "bid".to_string(),
                    price: 490000,
                    size: 2000,
                },
            ],
            asks: vec![
                PriceLevel {
                    side: "ask".to_string(),
                    price: 510000,
                    size: 500,
                },
                PriceLevel {
                    side: "ask".to_string(),
                    price: 520000,
                    size: 1500,
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
        assert_eq!(book.best_bid(), Some((500000, 1000)));
        assert_eq!(book.best_ask(), Some((510000, 500)));
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
                price: 500000,
                size: 1500, // Updated size
            }],
            asks: vec![PriceLevel {
                side: "ask".to_string(),
                price: 510000,
                size: 0, // Remove level
            }],
            is_snapshot: false,
            resync: false,
            message: None,
        };

        book.apply_delta(&delta).unwrap();

        assert_eq!(book.best_bid(), Some((500000, 1500)));
        assert_eq!(book.best_ask(), Some((520000, 1500)));
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

        assert_eq!(book.spread(), Some(10000));
        assert_eq!(book.midpoint(), Some(505000));
    }

    #[test]
    fn test_depth() {
        let mut book = LocalOrderbook::new("test".to_string());
        book.apply_snapshot(&create_snapshot());

        assert_eq!(book.total_bid_depth(), 3000);
        assert_eq!(book.total_ask_depth(), 2000);
    }
}
