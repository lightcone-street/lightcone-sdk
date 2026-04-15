//! Orderbook state containers — app-owned, SDK-provided update logic.

use crate::domain::orderbook::wire::OrderBook;
use crate::shared::OrderBookId;
use rust_decimal::Decimal;
use std::collections::BTreeMap;

/// Result of applying an orderbook snapshot or delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyResult {
    Applied,
    Ignored(IgnoreReason),
    RefreshRequired(RefreshReason),
}

/// A dropped update that does not require consumer action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoreReason {
    /// Deltas must have a positive sequence. Snapshots may have `seq == 0`.
    InvalidDeltaSequence { got: u64 },
    /// The delta arrived at or behind the current book sequence.
    StaleDelta { current: u64, got: u64 },
    /// The book is already waiting for a snapshot after a gap or resync signal.
    AlreadyAwaitingSnapshot { got: u64 },
}

/// A dropped update that means consumers should request a fresh snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshReason {
    /// A delta arrived before any snapshot initialized this book.
    MissingSnapshot { got: u64 },
    /// A delta skipped the next expected sequence.
    SequenceGap { expected: u64, got: u64 },
    /// The backend explicitly requested a resync.
    ServerResync { got: u64 },
}

/// Live orderbook state that can apply snapshots and deltas.
///
/// The app owns instances of this type (e.g. inside a Dioxus `Signal`).
/// The SDK provides the update methods.
#[derive(Debug, Clone, Default)]
pub struct OrderbookState {
    pub orderbook_id: OrderBookId,
    pub seq: u64,
    bids: BTreeMap<Decimal, Decimal>,
    asks: BTreeMap<Decimal, Decimal>,
    has_snapshot: bool,
    awaiting_snapshot: bool,
}

impl OrderbookState {
    pub fn new(orderbook_id: OrderBookId) -> Self {
        Self {
            orderbook_id,
            seq: 0,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            has_snapshot: false,
            awaiting_snapshot: false,
        }
    }

    /// Apply a WS orderbook message (snapshot replaces, delta merges).
    ///
    /// Server resync messages take precedence and return `RefreshRequired`.
    /// Otherwise, snapshots are applied and deltas with a `seq` at or below
    /// the current value are ignored to prevent stale or duplicate updates from
    /// corrupting the book. Deltas that skip one or more expected sequence
    /// values return `RefreshRequired` so callers can request a fresh snapshot
    /// instead of mutating a corrupted book.
    pub fn apply(&mut self, book: &OrderBook) -> ApplyResult {
        if book.resync {
            self.awaiting_snapshot = true;
            return ApplyResult::RefreshRequired(RefreshReason::ServerResync { got: book.seq });
        }

        if book.is_snapshot {
            self.bids.clear();
            self.asks.clear();
            self.has_snapshot = true;
            self.awaiting_snapshot = false;
        } else {
            if self.awaiting_snapshot {
                return ApplyResult::Ignored(IgnoreReason::AlreadyAwaitingSnapshot {
                    got: book.seq,
                });
            }

            // The backend sends snapshots with seq=0 and starts delta seq at 1.
            // A delta with seq=0 means it has no valid sequence, so drop it.
            if book.seq == 0 {
                return ApplyResult::Ignored(IgnoreReason::InvalidDeltaSequence { got: book.seq });
            }

            if !self.has_snapshot {
                self.awaiting_snapshot = true;
                return ApplyResult::RefreshRequired(RefreshReason::MissingSnapshot {
                    got: book.seq,
                });
            }

            if book.seq <= self.seq {
                return ApplyResult::Ignored(IgnoreReason::StaleDelta {
                    current: self.seq,
                    got: book.seq,
                });
            }

            let expected = self.seq + 1;
            if book.seq != expected {
                self.awaiting_snapshot = true;
                return ApplyResult::RefreshRequired(RefreshReason::SequenceGap {
                    expected,
                    got: book.seq,
                });
            }
        }

        self.seq = book.seq;

        for order in &book.bids {
            if order.size.is_zero() {
                self.bids.remove(&order.price);
            } else {
                self.bids.insert(order.price, order.size);
            }
        }

        for order in &book.asks {
            if order.size.is_zero() {
                self.asks.remove(&order.price);
            } else {
                self.asks.insert(order.price, order.size);
            }
        }

        ApplyResult::Applied
    }

    /// Bids sorted by price descending.
    pub fn bids(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.bids
    }

    /// Asks sorted by price ascending.
    pub fn asks(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.asks
    }

    /// Highest bid price.
    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().copied()
    }

    /// Lowest ask price.
    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().copied()
    }

    /// Mid price (average of best bid and best ask).
    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / Decimal::from(2)),
            _ => None,
        }
    }

    /// Spread between best ask and best bid.
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.seq = 0;
        self.has_snapshot = false;
        self.awaiting_snapshot = false;
    }
}

impl Default for OrderBookId {
    fn default() -> Self {
        OrderBookId::from("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::orderbook::wire::WsBookLevel;
    use crate::shared::Side;
    use rust_decimal::Decimal;

    fn order_book(
        snapshot: bool,
        seq: u64,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
    ) -> OrderBook {
        OrderBook {
            id: OrderBookId::from("ob_test"),
            is_snapshot: snapshot,
            seq,
            resync: false,
            bids: bids
                .into_iter()
                .map(|(price, size)| WsBookLevel {
                    side: Side::Bid,
                    price: Decimal::try_from(price).unwrap(),
                    size: Decimal::try_from(size).unwrap(),
                })
                .collect(),
            asks: asks
                .into_iter()
                .map(|(price, size)| WsBookLevel {
                    side: Side::Ask,
                    price: Decimal::try_from(price).unwrap(),
                    size: Decimal::try_from(size).unwrap(),
                })
                .collect(),
        }
    }

    #[test]
    fn test_snapshot_replaces_state() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.asks().len(), 1);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(50.0).unwrap()));
        assert_eq!(snap.best_ask(), Some(Decimal::try_from(51.0).unwrap()));

        assert_eq!(
            snap.apply(&order_book(true, 2, vec![(49.0, 20.0)], vec![(52.0, 8.0)])),
            ApplyResult::Applied
        );
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.asks().len(), 1);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(49.0).unwrap()));
        assert_eq!(snap.best_ask(), Some(Decimal::try_from(52.0).unwrap()));
    }

    #[test]
    fn test_delta_merges_with_snapshot() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );
        assert_eq!(
            snap.apply(&order_book(
                false,
                2,
                vec![(49.0, 15.0), (48.0, 3.0)],
                vec![(52.0, 2.0)],
            )),
            ApplyResult::Applied
        );
        assert_eq!(snap.bids().len(), 3);
        assert_eq!(snap.asks().len(), 2);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(50.0).unwrap()));
        assert_eq!(snap.best_ask(), Some(Decimal::try_from(51.0).unwrap()));
    }

    #[test]
    fn test_first_delta_after_zero_sequence_snapshot_applies() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 0, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );

        assert_eq!(
            snap.apply(&order_book(false, 1, vec![(49.0, 20.0)], vec![])),
            ApplyResult::Applied
        );

        assert_eq!(snap.seq, 1);
        assert_eq!(snap.bids().len(), 2);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(50.0).unwrap()));
    }

    #[test]
    fn test_resync_signal_leaves_book_unchanged() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );

        let mut resync = order_book(false, 2, vec![(49.0, 20.0)], vec![]);
        resync.resync = true;
        assert_eq!(
            snap.apply(&resync),
            ApplyResult::RefreshRequired(RefreshReason::ServerResync { got: 2 })
        );

        assert_eq!(snap.seq, 1);
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(50.0).unwrap()));

        assert_eq!(
            snap.apply(&order_book(false, 3, vec![(48.0, 20.0)], vec![])),
            ApplyResult::Ignored(IgnoreReason::AlreadyAwaitingSnapshot { got: 3 })
        );
    }

    #[test]
    fn test_zero_size_removes_level() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );
        assert_eq!(
            snap.apply(&order_book(false, 2, vec![(50.0, 0.0)], vec![])),
            ApplyResult::Applied
        );
        assert_eq!(snap.bids().len(), 0);
        assert_eq!(snap.best_bid(), None);
    }

    #[test]
    fn test_mid_price_and_spread() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(52.0, 5.0)])),
            ApplyResult::Applied
        );
        assert_eq!(snap.mid_price(), Some(Decimal::try_from(51.0).unwrap()));
        assert_eq!(snap.spread(), Some(Decimal::try_from(2.0).unwrap()));
    }

    #[test]
    fn test_stale_delta_is_dropped() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );
        assert_eq!(
            snap.apply(&order_book(false, 2, vec![(49.0, 20.0)], vec![])),
            ApplyResult::Applied
        );
        assert_eq!(snap.seq, 2);
        assert_eq!(snap.bids().len(), 2);

        // Stale delta (seq <= current) should be ignored
        assert_eq!(
            snap.apply(&order_book(false, 1, vec![(50.0, 0.0)], vec![])),
            ApplyResult::Ignored(IgnoreReason::StaleDelta { current: 2, got: 1 })
        );
        assert_eq!(snap.seq, 2);
        assert_eq!(snap.bids().len(), 2); // unchanged

        // Duplicate seq should also be ignored
        assert_eq!(
            snap.apply(&order_book(false, 2, vec![(50.0, 0.0)], vec![])),
            ApplyResult::Ignored(IgnoreReason::StaleDelta { current: 2, got: 2 })
        );
        assert_eq!(snap.bids().len(), 2); // unchanged

        // Snapshot always applies regardless of seq
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(48.0, 5.0)], vec![])),
            ApplyResult::Applied
        );
        assert_eq!(snap.seq, 1);
        assert_eq!(snap.bids().len(), 1);
    }

    #[test]
    fn test_gap_delta_is_detected_and_not_applied() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );

        assert_eq!(
            snap.apply(&order_book(false, 3, vec![(49.0, 20.0)], vec![])),
            ApplyResult::RefreshRequired(RefreshReason::SequenceGap {
                expected: 2,
                got: 3,
            })
        );
        assert_eq!(snap.seq, 1);
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(50.0).unwrap()));

        let mut resync = order_book(false, 4, vec![(48.0, 20.0)], vec![]);
        resync.resync = true;
        assert_eq!(
            snap.apply(&resync),
            ApplyResult::RefreshRequired(RefreshReason::ServerResync { got: 4 })
        );

        assert_eq!(
            snap.apply(&order_book(false, 5, vec![(47.0, 20.0)], vec![])),
            ApplyResult::Ignored(IgnoreReason::AlreadyAwaitingSnapshot { got: 5 })
        );
    }

    #[test]
    fn test_delta_before_snapshot_is_detected_as_gap() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));

        assert_eq!(
            snap.apply(&order_book(false, 1, vec![(50.0, 10.0)], vec![])),
            ApplyResult::RefreshRequired(RefreshReason::MissingSnapshot { got: 1 })
        );
        assert_eq!(snap.seq, 0);
        assert!(snap.is_empty());
    }

    #[test]
    fn test_zero_sequence_delta_is_ignored() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 0, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );

        assert_eq!(
            snap.apply(&order_book(false, 0, vec![(49.0, 20.0)], vec![])),
            ApplyResult::Ignored(IgnoreReason::InvalidDeltaSequence { got: 0 })
        );
        assert_eq!(snap.seq, 0);
        assert_eq!(snap.bids().len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );
        snap.clear();
        assert!(snap.is_empty());
        assert_eq!(snap.seq, 0);
    }
}
