//! Orderbook state containers — app-owned, SDK-provided update logic.
//!
//! The `book_update` stream is snapshot-only: every data frame carries the
//! full top-20 levels per side and replaces the previous book wholesale.
//! Equal and older revisions are discarded within a generation. Consumers
//! holding multiple aggregation views of one orderbook on the same connection
//! key their [`OrderbookState`] instances by
//! `(orderbook_id, aggregation)` using
//! [`OrderBook::aggregation`](crate::domain::orderbook::wire::OrderBook::aggregation).

use crate::domain::orderbook::{aggregation::BookAggregation, wire::OrderBook};
use crate::shared::OrderBookId;
use rust_decimal::Decimal;
use std::collections::BTreeMap;

/// Result of applying a WS orderbook frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyResult {
    Applied,
    DiscardedStale,
    SubscriptionMismatch,
    RefreshRequired(RefreshReason),
}

/// A dropped frame that means consumers should refresh the subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshReason {
    /// The backend explicitly requested a resync: unsubscribe and
    /// re-subscribe with the same parameters (including aggregation) to
    /// receive a fresh snapshot.
    ServerResync,
}

/// Live orderbook state replaced wholesale by snapshot frames.
///
/// The app owns instances of this type (e.g. inside a Dioxus `Signal`).
/// The SDK provides the update methods.
#[derive(Debug, Clone, Default)]
pub struct OrderbookState {
    pub orderbook_id: OrderBookId,
    pub aggregation: BookAggregation,
    /// Last accepted engine depth revision. Forward gaps are valid.
    pub seq: u64,
    last_seq: Option<u64>,
    pub bids_truncated: bool,
    pub asks_truncated: bool,
    bids: BTreeMap<Decimal, Decimal>,
    asks: BTreeMap<Decimal, Decimal>,
}

impl OrderbookState {
    pub fn new(orderbook_id: OrderBookId) -> Self {
        Self::with_aggregation(orderbook_id, BookAggregation::FULL)
    }

    pub fn with_aggregation(orderbook_id: OrderBookId, aggregation: BookAggregation) -> Self {
        Self {
            orderbook_id,
            aggregation: aggregation.normalized(),
            seq: 0,
            last_seq: None,
            bids_truncated: false,
            asks_truncated: false,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Apply a full WS snapshot when its revision is newer in this generation.
    ///
    /// `resync` frames take precedence and leave the book untouched — the
    /// caller must re-subscribe with the same parameters. Every accepted data
    /// frame replaces the book wholesale; gaps are normal and do not resync.
    pub fn apply(&mut self, book: &OrderBook) -> ApplyResult {
        if book.id != self.orderbook_id || book.aggregation() != self.aggregation {
            return ApplyResult::SubscriptionMismatch;
        }
        if book.resync {
            return ApplyResult::RefreshRequired(RefreshReason::ServerResync);
        }
        if self.last_seq.is_some_and(|last| book.seq <= last) {
            return ApplyResult::DiscardedStale;
        }

        self.bids.clear();
        self.asks.clear();

        for order in &book.bids {
            if !order.size.is_zero() {
                self.bids.insert(order.price, order.size);
            }
        }
        for order in &book.asks {
            if !order.size.is_zero() {
                self.asks.insert(order.price, order.size);
            }
        }
        self.seq = book.seq;
        self.last_seq = Some(book.seq);
        self.bids_truncated = book.bids_truncated;
        self.asks_truncated = book.asks_truncated;

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

    /// Start a reconnect/resubscribe generation. Existing levels remain visible
    /// until the fresh initial snapshot is accepted.
    pub fn begin_generation(&mut self) {
        self.last_seq = None;
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.seq = 0;
        self.last_seq = None;
        self.bids_truncated = false;
        self.asks_truncated = false;
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
            id: OrderBookId::from("ob1"),
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
            bids_truncated: false,
            asks_truncated: false,
            timestamp: None,
            n_sig_figs: None,
            mantissa: None,
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
    fn test_lower_and_equal_revisions_are_discarded() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 42, vec![(50.0, 10.0)], vec![])),
            ApplyResult::Applied
        );
        assert_eq!(snap.seq, 42);

        assert_eq!(
            snap.apply(&order_book(true, 7, vec![(49.0, 20.0)], vec![])),
            ApplyResult::DiscardedStale
        );
        assert_eq!(
            snap.apply(&order_book(true, 42, vec![], vec![])),
            ApplyResult::DiscardedStale
        );
        assert_eq!(snap.seq, 42);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(50.0).unwrap()));
    }

    #[test]
    fn test_lower_fresh_revision_applies_after_resubscribe() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 42, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );

        let mut resync = order_book(false, 0, vec![], vec![]);
        resync.resync = true;
        assert_eq!(
            snap.apply(&resync),
            ApplyResult::RefreshRequired(RefreshReason::ServerResync)
        );
        // Resync leaves the book untouched.
        assert_eq!(snap.seq, 42);
        assert_eq!(snap.bids().len(), 1);

        // A lower revision is valid in a fresh subscription generation.
        snap.begin_generation();
        assert_eq!(
            snap.apply(&order_book(true, 7, vec![(48.0, 5.0)], vec![(52.0, 2.0)])),
            ApplyResult::Applied
        );
        assert_eq!(snap.seq, 7);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(48.0).unwrap()));
        assert_eq!(snap.best_ask(), Some(Decimal::try_from(52.0).unwrap()));
    }

    #[test]
    fn test_forward_gap_and_truncation_metadata_apply() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 10, vec![], vec![])),
            ApplyResult::Applied
        );
        let mut next = order_book(true, 15, vec![(50.0, 1.0)], vec![]);
        next.bids_truncated = true;
        assert_eq!(snap.apply(&next), ApplyResult::Applied);
        assert_eq!(snap.seq, 15);
        assert!(snap.bids_truncated);
        assert!(!snap.asks_truncated);
    }

    #[test]
    fn test_aggregation_generations_are_separate() {
        let mut full = OrderbookState::new(OrderBookId::from("ob1"));
        let aggregation = BookAggregation::validate(Some(5), Some(2)).unwrap();
        let mut grouped = OrderbookState::with_aggregation(OrderBookId::from("ob1"), aggregation);
        let full_frame = order_book(true, 100, vec![], vec![]);
        let mut grouped_frame = order_book(true, 3, vec![], vec![]);
        grouped_frame.n_sig_figs = Some(5);
        grouped_frame.mantissa = Some(2);
        assert_eq!(full.apply(&full_frame), ApplyResult::Applied);
        assert_eq!(grouped.apply(&grouped_frame), ApplyResult::Applied);
        assert_eq!(
            full.apply(&grouped_frame),
            ApplyResult::SubscriptionMismatch
        );
        assert_eq!(full.seq, 100);
        assert_eq!(grouped.seq, 3);
    }

    #[test]
    fn test_data_frames_replace_regardless_of_snapshot_flag() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );

        // Every non-resync data frame is a snapshot by contract — the
        // is_snapshot flag is not consulted, so a server omitting it cannot
        // freeze the book.
        assert_eq!(
            snap.apply(&order_book(false, 2, vec![(49.0, 20.0)], vec![])),
            ApplyResult::Applied
        );
        assert_eq!(snap.seq, 2);
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.asks().len(), 0);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(49.0).unwrap()));
    }

    #[test]
    fn test_zero_size_levels_are_skipped() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(
                true,
                1,
                vec![(50.0, 10.0), (49.0, 0.0)],
                vec![(51.0, 5.0)],
            )),
            ApplyResult::Applied
        );
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(50.0).unwrap()));
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
