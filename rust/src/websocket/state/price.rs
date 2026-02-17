//! Price history state management.
//!
//! Maintains local state for price history candles.

use std::collections::HashMap;

use crate::shared::Resolution;
use crate::websocket::types::{Candle, PriceHistoryData};

/// Key for price history subscriptions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PriceHistoryKey {
    pub orderbook_id: String,
    pub resolution: String,
}

impl PriceHistoryKey {
    pub fn new(orderbook_id: String, resolution: String) -> Self {
        Self {
            orderbook_id,
            resolution,
        }
    }
}

/// Price history state for a single orderbook/resolution pair
#[derive(Debug, Clone)]
pub struct PriceHistory {
    /// Orderbook identifier
    pub orderbook_id: String,
    /// Resolution (1m, 5m, 15m, 1h, 4h, 1d)
    pub resolution: String,
    /// Whether OHLCV data is included
    pub include_ohlcv: bool,
    /// Candles sorted by timestamp (newest first)
    candles: Vec<Candle>,
    /// Index by timestamp for fast lookup
    candle_index: HashMap<i64, usize>,
    /// Last server timestamp
    last_timestamp: Option<i64>,
    /// Server time from last message
    server_time: Option<i64>,
    /// Whether initial snapshot has been received
    has_snapshot: bool,
}

impl PriceHistory {
    /// Create a new empty price history
    pub fn new(orderbook_id: String, resolution: String, include_ohlcv: bool) -> Self {
        Self {
            orderbook_id,
            resolution,
            include_ohlcv,
            candles: Vec::new(),
            candle_index: HashMap::new(),
            last_timestamp: None,
            server_time: None,
            has_snapshot: false,
        }
    }

    /// Apply a snapshot (historical candles)
    pub fn apply_snapshot(&mut self, data: &PriceHistoryData) {
        // Clear existing state
        self.candles.clear();
        self.candle_index.clear();

        // Backend sends candles oldest-first (chronological); reverse to newest-first
        for candle in data.prices.iter().rev() {
            let idx = self.candles.len();
            self.candle_index.insert(candle.t, idx);
            self.candles.push(candle.clone());
        }

        self.last_timestamp = data.last_timestamp;
        self.server_time = data.server_time;
        self.has_snapshot = true;

        // Update include_ohlcv if provided
        if let Some(include_ohlcv) = data.include_ohlcv {
            self.include_ohlcv = include_ohlcv;
        }
    }

    /// Apply an update (new or updated candle)
    pub fn apply_update(&mut self, data: &PriceHistoryData) {
        if let Some(candle) = data.to_candle() {
            self.update_or_append_candle(candle);
        }
    }

    /// Update an existing candle or append a new one
    fn update_or_append_candle(&mut self, candle: Candle) {
        if let Some(&idx) = self.candle_index.get(&candle.t) {
            // Update existing candle
            self.candles[idx] = candle;
        } else {
            // New candle - insert at the correct position (newest first)
            let insert_pos = self
                .candles
                .iter()
                .position(|c| c.t < candle.t)
                .unwrap_or(self.candles.len());

            // Update indices for candles that will shift
            for (ts, idx) in self.candle_index.iter_mut() {
                if *idx >= insert_pos {
                    *idx += 1;
                }
                let _ = ts;
            }

            self.candle_index.insert(candle.t, insert_pos);
            self.candles.insert(insert_pos, candle);

            // Trim to max 1000 candles
            while self.candles.len() > 1000 {
                if let Some(removed) = self.candles.pop() {
                    self.candle_index.remove(&removed.t);
                }
            }
        }

        // Update last timestamp
        if let Some(first) = self.candles.first() {
            self.last_timestamp = Some(first.t);
        }
    }

    /// Handle heartbeat (update server time)
    pub fn apply_heartbeat(&mut self, data: &PriceHistoryData) {
        self.server_time = data.server_time;
    }

    /// Apply any price history event
    pub fn apply_event(&mut self, data: &PriceHistoryData) {
        match data.event_type.as_str() {
            "snapshot" => self.apply_snapshot(data),
            "update" => self.apply_update(data),
            "heartbeat" => self.apply_heartbeat(data),
            _ => {
                tracing::warn!("Unknown price history event type: {}", data.event_type);
            }
        }
    }

    /// Get all candles (newest first)
    pub fn candles(&self) -> &[Candle] {
        &self.candles
    }

    /// Get the N most recent candles
    pub fn recent_candles(&self, n: usize) -> &[Candle] {
        let end = n.min(self.candles.len());
        &self.candles[..end]
    }

    /// Get a candle by timestamp
    pub fn get_candle(&self, timestamp: i64) -> Option<&Candle> {
        self.candle_index.get(&timestamp).map(|&idx| &self.candles[idx])
    }

    /// Get the most recent candle
    pub fn latest_candle(&self) -> Option<&Candle> {
        self.candles.first()
    }

    /// Get the oldest candle
    pub fn oldest_candle(&self) -> Option<&Candle> {
        self.candles.last()
    }

    /// Get current midpoint price (from most recent candle)
    pub fn current_midpoint(&self) -> Option<String> {
        self.candles.first().and_then(|c| c.m.clone())
    }

    /// Get current best bid (from most recent candle)
    pub fn current_best_bid(&self) -> Option<String> {
        self.candles.first().and_then(|c| c.bb.clone())
    }

    /// Get current best ask (from most recent candle)
    pub fn current_best_ask(&self) -> Option<String> {
        self.candles.first().and_then(|c| c.ba.clone())
    }

    /// Number of candles
    pub fn candle_count(&self) -> usize {
        self.candles.len()
    }

    /// Whether the price history has received its initial snapshot
    pub fn has_snapshot(&self) -> bool {
        self.has_snapshot
    }

    /// Last candle timestamp
    pub fn last_timestamp(&self) -> Option<i64> {
        self.last_timestamp
    }

    /// Server time from last message
    pub fn server_time(&self) -> Option<i64> {
        self.server_time
    }

    /// Get resolution as enum
    pub fn resolution_enum(&self) -> Option<Resolution> {
        match self.resolution.as_str() {
            "1m" => Some(Resolution::OneMinute),
            "5m" => Some(Resolution::FiveMinutes),
            "15m" => Some(Resolution::FifteenMinutes),
            "1h" => Some(Resolution::OneHour),
            "4h" => Some(Resolution::FourHours),
            "1d" => Some(Resolution::OneDay),
            _ => None,
        }
    }

    /// Clear the price history (for disconnect/resync)
    pub fn clear(&mut self) {
        self.candles.clear();
        self.candle_index.clear();
        self.last_timestamp = None;
        self.server_time = None;
        self.has_snapshot = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_snapshot() -> PriceHistoryData {
        PriceHistoryData {
            event_type: "snapshot".to_string(),
            orderbook_id: Some("ob1".to_string()),
            resolution: Some("1m".to_string()),
            include_ohlcv: Some(true),
            // Backend sends candles oldest-first (chronological order)
            prices: vec![
                Candle {
                    t: 1704067200000, // Older (sent first by backend)
                    o: Some("0.500000".to_string()),
                    h: Some("0.510000".to_string()),
                    l: Some("0.495000".to_string()),
                    c: Some("0.505000".to_string()),
                    v: Some("0.010000".to_string()),
                    m: Some("0.502500".to_string()),
                    bb: Some("0.500000".to_string()),
                    ba: Some("0.505000".to_string()),
                },
                Candle {
                    t: 1704067260000, // Newer (sent second by backend)
                    o: Some("0.505000".to_string()),
                    h: Some("0.508000".to_string()),
                    l: Some("0.503000".to_string()),
                    c: Some("0.507000".to_string()),
                    v: Some("0.005000".to_string()),
                    m: Some("0.505500".to_string()),
                    bb: Some("0.505000".to_string()),
                    ba: Some("0.506000".to_string()),
                },
            ],
            last_timestamp: Some(1704067260000),
            server_time: Some(1704067320000),
            last_processed: None,
            t: None,
            o: None,
            h: None,
            l: None,
            c: None,
            v: None,
            m: None,
            bb: None,
            ba: None,
        }
    }

    #[test]
    fn test_apply_snapshot() {
        let mut history = PriceHistory::new("ob1".to_string(), "1m".to_string(), true);
        let snapshot = create_snapshot();

        history.apply_snapshot(&snapshot);

        assert!(history.has_snapshot());
        assert_eq!(history.candle_count(), 2);
        assert_eq!(history.last_timestamp(), Some(1704067260000));
        assert_eq!(history.server_time(), Some(1704067320000));
    }

    #[test]
    fn test_candle_order() {
        let mut history = PriceHistory::new("ob1".to_string(), "1m".to_string(), true);
        history.apply_snapshot(&create_snapshot());

        let candles = history.candles();
        assert_eq!(candles[0].t, 1704067260000); // Newer first
        assert_eq!(candles[1].t, 1704067200000); // Older second
    }

    #[test]
    fn test_apply_update_existing() {
        let mut history = PriceHistory::new("ob1".to_string(), "1m".to_string(), true);
        history.apply_snapshot(&create_snapshot());

        let update = PriceHistoryData {
            event_type: "update".to_string(),
            orderbook_id: Some("ob1".to_string()),
            resolution: Some("1m".to_string()),
            include_ohlcv: None,
            prices: vec![],
            last_timestamp: None,
            server_time: None,
            last_processed: None,
            t: Some(1704067260000),
            o: Some("0.505000".to_string()),
            h: Some("0.510000".to_string()), // Updated high
            l: Some("0.503000".to_string()),
            c: Some("0.509000".to_string()), // Updated close
            v: Some("0.006000".to_string()), // Updated volume
            m: Some("0.507000".to_string()),
            bb: Some("0.506000".to_string()),
            ba: Some("0.508000".to_string()),
        };

        history.apply_update(&update);

        let candle = history.get_candle(1704067260000).unwrap();
        assert_eq!(candle.h, Some("0.510000".to_string()));
        assert_eq!(candle.c, Some("0.509000".to_string()));
        assert_eq!(candle.v, Some("0.006000".to_string()));
    }

    #[test]
    fn test_apply_update_new_candle() {
        let mut history = PriceHistory::new("ob1".to_string(), "1m".to_string(), true);
        history.apply_snapshot(&create_snapshot());

        let update = PriceHistoryData {
            event_type: "update".to_string(),
            orderbook_id: Some("ob1".to_string()),
            resolution: Some("1m".to_string()),
            include_ohlcv: None,
            prices: vec![],
            last_timestamp: None,
            server_time: None,
            last_processed: None,
            t: Some(1704067320000), // New timestamp
            o: Some("0.507000".to_string()),
            h: Some("0.512000".to_string()),
            l: Some("0.506000".to_string()),
            c: Some("0.511000".to_string()),
            v: Some("0.008000".to_string()),
            m: Some("0.509000".to_string()),
            bb: Some("0.508000".to_string()),
            ba: Some("0.510000".to_string()),
        };

        history.apply_update(&update);

        assert_eq!(history.candle_count(), 3);
        assert_eq!(history.latest_candle().unwrap().t, 1704067320000);
    }

    #[test]
    fn test_current_prices() {
        let mut history = PriceHistory::new("ob1".to_string(), "1m".to_string(), true);
        history.apply_snapshot(&create_snapshot());

        assert_eq!(history.current_midpoint(), Some("0.505500".to_string()));
        assert_eq!(history.current_best_bid(), Some("0.505000".to_string()));
        assert_eq!(history.current_best_ask(), Some("0.506000".to_string()));
    }
}
